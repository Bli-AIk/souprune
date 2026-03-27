use crate::config::{BindingTable, PropertyMode, PropertyTable};
use anyhow::{Result, bail};
use serde::Serialize;
use souprune_schema::Val;
use souprune_schema::view::{
    SerializableTransform, TextAlignDef, TextAnchorDef, TextDef, ViewFontDef, ViewLayoutAsset,
    ViewNodeDef,
};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct ConcreteTextParameters {
    pub font: ViewFontDef,
    pub align: TextAlignDef,
    pub anchor: TextAnchorDef,
    pub translation_x: f32,
    pub translation_y: f32,
    pub world_scale_x: f32,
    pub world_scale_y: f32,
    pub line_height: f32,
    pub char_spacing: f32,
    pub word_spacing: f32,
}

impl Default for ConcreteTextParameters {
    fn default() -> Self {
        Self {
            font: ViewFontDef::DeterminationMono,
            align: TextAlignDef::Left,
            anchor: TextAnchorDef::BottomRight,
            translation_x: 0.0,
            translation_y: 0.0,
            world_scale_x: 13.0,
            world_scale_y: 13.0,
            line_height: 1.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandidateSearchPlan {
    domains: CandidateDomains,
    full_search_space: usize,
    target_similarity: f32,
    evaluation_budget: usize,
    population_size: usize,
    elite_count: usize,
    seed_genome: CandidateGenome,
    best_genome: CandidateGenome,
    best_fitness: f32,
    generation_improved: bool,
    generations_without_improvement: usize,
    generation: usize,
    evaluations_done: usize,
    next_population_index: usize,
    population: Vec<CandidateGenome>,
    population_scores: Vec<Option<f32>>,
    last_issued_slot: Option<usize>,
    rng: SimpleRng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchParameterField {
    Font,
    Align,
    Anchor,
    TranslationX,
    TranslationY,
    WorldScaleX,
    WorldScaleY,
    LineHeight,
    CharSpacing,
    WordSpacing,
}

#[derive(Debug, Clone)]
struct CandidateDomains {
    fonts: Vec<ViewFontDef>,
    aligns: Vec<TextAlignDef>,
    anchors: Vec<TextAnchorDef>,
    translation_xs: Vec<f32>,
    translation_ys: Vec<f32>,
    world_scale_xs: Vec<f32>,
    world_scale_ys: Vec<f32>,
    world_scale_bound: bool,
    line_heights: Vec<f32>,
    char_spacings: Vec<f32>,
    word_spacings: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CandidateGenome {
    font: usize,
    align: usize,
    anchor: usize,
    translation_x: usize,
    translation_y: usize,
    world_scale_x: usize,
    world_scale_y: usize,
    line_height: usize,
    char_spacing: usize,
    word_spacing: usize,
}

#[derive(Debug, Clone)]
struct RankedGenome {
    genome: CandidateGenome,
    fitness: f32,
}

#[derive(Debug, Clone, Copy)]
struct SimpleRng {
    state: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchPhase {
    Explore,
    Focus,
    Refine,
    Polish,
}

impl CandidateSearchPlan {
    pub fn build(
        assume_single_line: bool,
        properties: &PropertyTable,
        bindings: &BindingTable,
        defaults: ConcreteTextParameters,
        target_similarity: f32,
        evaluation_budget_override: Option<usize>,
        population_size_override: Option<usize>,
    ) -> Result<Self> {
        let fonts = properties.font.resolve_font_values(defaults.font.clone())?;
        let aligns = properties.align.resolve_align_values(defaults.align)?;
        let anchors = properties.anchor.resolve_anchor_values(defaults.anchor)?;
        let translation_xs = properties
            .translation_x
            .resolve_values("translation_x", defaults.translation_x)?;
        let translation_ys = properties
            .translation_y
            .resolve_values("translation_y", defaults.translation_y)?;
        let mut world_scale_xs = properties
            .world_scale_x
            .resolve_values("world_scale_x", defaults.world_scale_x)?;
        let mut world_scale_ys = properties
            .world_scale_y
            .resolve_values("world_scale_y", defaults.world_scale_y)?;
        let mut line_heights = properties
            .line_height
            .resolve_values("line_height", defaults.line_height)?;
        let char_spacings = properties
            .char_spacing
            .resolve_values("char_spacing", defaults.char_spacing)?;
        let word_spacings = properties
            .word_spacing
            .resolve_values("word_spacing", defaults.word_spacing)?;

        let world_scale_bound = bindings.world_scale.is_bound();
        if world_scale_bound {
            let bound_world_scales = resolve_bound_numeric_values(
                "world_scale",
                properties.world_scale_x.mode,
                &world_scale_xs,
                properties.world_scale_y.mode,
                &world_scale_ys,
            )?;
            world_scale_xs = bound_world_scales.clone();
            world_scale_ys = bound_world_scales;
        }

        if assume_single_line && !line_heights.is_empty() {
            line_heights.truncate(1);
        }

        let dimensions = [
            fonts.len(),
            aligns.len(),
            anchors.len(),
            translation_xs.len(),
            translation_ys.len(),
            world_scale_xs.len(),
            if world_scale_bound {
                1
            } else {
                world_scale_ys.len()
            },
            line_heights.len(),
            char_spacings.len(),
            word_spacings.len(),
        ];
        let full_search_space = dimensions.into_iter().try_fold(1usize, |acc, len| {
            acc.checked_mul(len)
                .ok_or_else(|| anyhow::anyhow!("search space overflowed usize"))
        })?;
        if full_search_space == 0 {
            bail!("search space must contain at least one candidate");
        }

        let domains = CandidateDomains {
            fonts,
            aligns,
            anchors,
            translation_xs,
            translation_ys,
            world_scale_xs,
            world_scale_ys,
            world_scale_bound,
            line_heights,
            char_spacings,
            word_spacings,
        };
        let seed_genome = domains.default_genome(&defaults);
        let evaluation_budget = evaluation_budget_override
            .unwrap_or_else(|| max_evaluation_budget(&domains))
            .max(1)
            .min(full_search_space);
        let population_size = population_size_override
            .unwrap_or_else(|| default_population_size(&domains))
            .max(1)
            .min(full_search_space)
            .min(evaluation_budget);
        let elite_count = population_size.clamp(1, 6);
        let rng = SimpleRng::from_entropy();

        Ok(Self {
            domains,
            full_search_space,
            target_similarity,
            evaluation_budget,
            population_size,
            elite_count,
            seed_genome,
            best_genome: seed_genome,
            best_fitness: f32::NEG_INFINITY,
            generation_improved: false,
            generations_without_improvement: 0,
            generation: 0,
            evaluations_done: 0,
            next_population_index: 0,
            population: Vec::new(),
            population_scores: Vec::new(),
            last_issued_slot: None,
            rng,
        })
    }

    pub fn total_candidates(&self) -> usize {
        self.evaluation_budget
    }

    pub fn seed_parameters(&self) -> ConcreteTextParameters {
        self.domains.parameters_from_genome(self.seed_genome)
    }

    pub fn constrain_parameters(&self, parameters: &mut ConcreteTextParameters) {
        *parameters = self
            .domains
            .parameters_from_genome(self.domains.default_genome(parameters));
    }

    pub fn nudge_parameter(
        &self,
        parameters: &mut ConcreteTextParameters,
        field: SearchParameterField,
        steps: i32,
    ) {
        if steps == 0 {
            return;
        }

        let mut genome = self.domains.default_genome(parameters);
        match field {
            SearchParameterField::Font => {
                genome.font = clamp_gene_index(genome.font, steps, self.domains.fonts.len());
            }
            SearchParameterField::Align => {
                genome.align = clamp_gene_index(genome.align, steps, self.domains.aligns.len());
            }
            SearchParameterField::Anchor => {
                genome.anchor = clamp_gene_index(genome.anchor, steps, self.domains.anchors.len());
            }
            SearchParameterField::TranslationX => {
                genome.translation_x = clamp_gene_index(
                    genome.translation_x,
                    steps,
                    self.domains.translation_xs.len(),
                );
            }
            SearchParameterField::TranslationY => {
                genome.translation_y = clamp_gene_index(
                    genome.translation_y,
                    steps,
                    self.domains.translation_ys.len(),
                );
            }
            SearchParameterField::WorldScaleX => {
                genome.world_scale_x = clamp_gene_index(
                    genome.world_scale_x,
                    steps,
                    self.domains.world_scale_xs.len(),
                );
                if self.domains.world_scale_bound {
                    genome.world_scale_y = genome.world_scale_x;
                }
            }
            SearchParameterField::WorldScaleY => {
                if self.domains.world_scale_bound {
                    genome.world_scale_x = clamp_gene_index(
                        genome.world_scale_x,
                        steps,
                        self.domains.world_scale_xs.len(),
                    );
                    genome.world_scale_y = genome.world_scale_x;
                } else {
                    genome.world_scale_y = clamp_gene_index(
                        genome.world_scale_y,
                        steps,
                        self.domains.world_scale_ys.len(),
                    );
                }
            }
            SearchParameterField::LineHeight => {
                genome.line_height =
                    clamp_gene_index(genome.line_height, steps, self.domains.line_heights.len());
            }
            SearchParameterField::CharSpacing => {
                genome.char_spacing =
                    clamp_gene_index(genome.char_spacing, steps, self.domains.char_spacings.len());
            }
            SearchParameterField::WordSpacing => {
                genome.word_spacing =
                    clamp_gene_index(genome.word_spacing, steps, self.domains.word_spacings.len());
            }
        }

        *parameters = self.domains.parameters_from_genome(genome);
    }

    pub fn record_fitness(&mut self, fitness: f32) {
        let Some(slot) = self.last_issued_slot.take() else {
            return;
        };
        if let Some(score_slot) = self.population_scores.get_mut(slot) {
            *score_slot = Some(fitness);
        }
        let genome = self.population[slot];
        if fitness > self.best_fitness + 0.0001 {
            self.best_fitness = fitness;
            self.best_genome = genome;
            self.generation_improved = true;
            self.generations_without_improvement = 0;
        }
    }

    pub fn next_candidate(&mut self) -> Option<(usize, ConcreteTextParameters)> {
        if self.evaluations_done >= self.evaluation_budget {
            return None;
        }

        if self.population.is_empty() || self.next_population_index >= self.population.len() {
            self.prepare_next_population();
        }
        if self.population.is_empty() {
            return None;
        }

        let evaluation_index = self.evaluations_done;
        let slot = self.next_population_index;
        self.next_population_index += 1;
        self.evaluations_done += 1;
        self.last_issued_slot = Some(slot);

        Some((
            evaluation_index,
            self.domains.parameters_from_genome(self.population[slot]),
        ))
    }

    pub fn restart_from_parameters(
        &mut self,
        parameters: &ConcreteTextParameters,
    ) -> (usize, ConcreteTextParameters) {
        self.seed_genome = self.domains.default_genome(parameters);
        self.best_genome = self.seed_genome;
        self.reset_search_state();
        self.next_candidate()
            .expect("search plan always contains at least one candidate")
    }

    fn reset_search_state(&mut self) {
        self.best_fitness = f32::NEG_INFINITY;
        self.generation_improved = false;
        self.generations_without_improvement = 0;
        self.generation = 0;
        self.evaluations_done = 0;
        self.next_population_index = 0;
        self.population.clear();
        self.population_scores.clear();
        self.last_issued_slot = None;
        self.rng = SimpleRng::from_entropy();
    }
}

impl CandidateDomains {
    fn parameters_from_genome(&self, genome: CandidateGenome) -> ConcreteTextParameters {
        ConcreteTextParameters {
            font: self.fonts[genome.font].clone(),
            align: self.aligns[genome.align],
            anchor: self.anchors[genome.anchor],
            translation_x: self.translation_xs[genome.translation_x],
            translation_y: self.translation_ys[genome.translation_y],
            world_scale_x: self.world_scale_xs[genome.world_scale_x],
            world_scale_y: self.world_scale_ys[if self.world_scale_bound {
                genome.world_scale_x
            } else {
                genome.world_scale_y
            }],
            line_height: self.line_heights[genome.line_height],
            char_spacing: self.char_spacings[genome.char_spacing],
            word_spacing: self.word_spacings[genome.word_spacing],
        }
    }

    fn default_genome(&self, defaults: &ConcreteTextParameters) -> CandidateGenome {
        let world_scale_x = find_nearest_f32_index(&self.world_scale_xs, defaults.world_scale_x);
        CandidateGenome {
            font: find_exact_index(&self.fonts, &defaults.font),
            align: find_exact_index(&self.aligns, &defaults.align),
            anchor: find_exact_index(&self.anchors, &defaults.anchor),
            translation_x: find_nearest_f32_index(&self.translation_xs, defaults.translation_x),
            translation_y: find_nearest_f32_index(&self.translation_ys, defaults.translation_y),
            world_scale_x,
            world_scale_y: if self.world_scale_bound {
                world_scale_x
            } else {
                find_nearest_f32_index(&self.world_scale_ys, defaults.world_scale_y)
            },
            line_height: find_nearest_f32_index(&self.line_heights, defaults.line_height),
            char_spacing: find_nearest_f32_index(&self.char_spacings, defaults.char_spacing),
            word_spacing: find_nearest_f32_index(&self.word_spacings, defaults.word_spacing),
        }
    }
}

impl CandidateSearchPlan {
    fn prepare_next_population(&mut self) {
        if self.evaluations_done == 0 || self.population.is_empty() {
            self.seed_initial_population();
            return;
        }

        let ranked = self.rank_population();
        if ranked.is_empty() {
            self.seed_initial_population();
            return;
        }

        if !self.generation_improved {
            self.generations_without_improvement += 1;
        }
        self.generation_improved = false;
        self.generation += 1;
        self.next_population_index = 0;
        self.last_issued_slot = None;

        let remaining_budget = self.evaluation_budget.saturating_sub(self.evaluations_done);
        let target_population_size = self.population_size.min(remaining_budget).max(1);
        let phase = self.current_phase();

        let mut next_population = Vec::with_capacity(target_population_size);
        let mut unique = HashSet::with_capacity(target_population_size);

        for ranked_genome in ranked
            .iter()
            .take(self.elite_count.min(target_population_size))
        {
            if unique.insert(ranked_genome.genome) {
                next_population.push(ranked_genome.genome);
            }
        }

        for candidate in self.local_neighbor_candidates(self.best_genome, phase) {
            if next_population.len() >= target_population_size {
                break;
            }
            if unique.insert(candidate) {
                next_population.push(candidate);
            }
        }
        if self.generations_without_improvement >= 3 {
            for candidate in self.local_neighbor_candidates(self.seed_genome, SearchPhase::Focus) {
                if next_population.len() >= target_population_size {
                    break;
                }
                if unique.insert(candidate) {
                    next_population.push(candidate);
                }
            }
        }

        let mut attempts = 0usize;
        let max_attempts = target_population_size.saturating_mul(32).max(64);
        let random_candidate_rate = self.random_candidate_rate(phase);
        while next_population.len() < target_population_size
            && unique.len() < self.full_search_space
            && attempts < max_attempts
        {
            let candidate = if self.rng.next_f32() < random_candidate_rate {
                self.random_genome()
            } else {
                self.breed_genome(&ranked, phase)
            };
            attempts += 1;
            if unique.insert(candidate) {
                next_population.push(candidate);
            }
        }

        while next_population.len() < target_population_size
            && unique.len() < self.full_search_space
        {
            let candidate = self.random_genome();
            if unique.insert(candidate) {
                next_population.push(candidate);
            }
        }

        self.population = next_population;
        self.population_scores = vec![None; self.population.len()];
    }

    fn seed_initial_population(&mut self) {
        self.generation = 0;
        self.next_population_index = 0;
        self.last_issued_slot = None;
        self.generation_improved = false;
        self.generations_without_improvement = 0;

        let remaining_budget = self.evaluation_budget.saturating_sub(self.evaluations_done);
        let target_population_size = self.population_size.min(remaining_budget).max(1);

        let mut population = Vec::with_capacity(target_population_size);
        let mut unique = HashSet::with_capacity(target_population_size);

        unique.insert(self.seed_genome);
        population.push(self.seed_genome);

        for candidate in self.seed_neighbor_candidates() {
            if population.len() >= target_population_size {
                break;
            }
            if unique.insert(candidate) {
                population.push(candidate);
            }
        }

        while population.len() < target_population_size && unique.len() < self.full_search_space {
            let genome = self.random_genome();
            if unique.insert(genome) {
                population.push(genome);
            }
        }

        self.population = population;
        self.population_scores = vec![None; self.population.len()];
    }

    fn rank_population(&self) -> Vec<RankedGenome> {
        let mut ranked = self
            .population
            .iter()
            .copied()
            .zip(self.population_scores.iter().copied())
            .filter_map(|(genome, fitness)| fitness.map(|fitness| RankedGenome { genome, fitness }))
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| right.fitness.total_cmp(&left.fitness));
        ranked
    }

    fn random_genome(&mut self) -> CandidateGenome {
        let world_scale_x = self.rng.next_usize(self.domains.world_scale_xs.len());
        CandidateGenome {
            font: self.rng.next_usize(self.domains.fonts.len()),
            align: self.rng.next_usize(self.domains.aligns.len()),
            anchor: self.rng.next_usize(self.domains.anchors.len()),
            translation_x: self.rng.next_usize(self.domains.translation_xs.len()),
            translation_y: self.rng.next_usize(self.domains.translation_ys.len()),
            world_scale_x,
            world_scale_y: if self.domains.world_scale_bound {
                world_scale_x
            } else {
                self.rng.next_usize(self.domains.world_scale_ys.len())
            },
            line_height: self.rng.next_usize(self.domains.line_heights.len()),
            char_spacing: self.rng.next_usize(self.domains.char_spacings.len()),
            word_spacing: self.rng.next_usize(self.domains.word_spacings.len()),
        }
    }

    fn breed_genome(&mut self, ranked: &[RankedGenome], phase: SearchPhase) -> CandidateGenome {
        let parent_a = self.select_parent(ranked, phase);
        let parent_b = self.select_parent(ranked, phase);
        let mut child = CandidateGenome {
            font: self.mix_gene(parent_a.font, parent_b.font),
            align: self.mix_gene(parent_a.align, parent_b.align),
            anchor: self.mix_gene(parent_a.anchor, parent_b.anchor),
            translation_x: self.mix_gene(parent_a.translation_x, parent_b.translation_x),
            translation_y: self.mix_gene(parent_a.translation_y, parent_b.translation_y),
            world_scale_x: self.mix_gene(parent_a.world_scale_x, parent_b.world_scale_x),
            world_scale_y: self.mix_gene(parent_a.world_scale_y, parent_b.world_scale_y),
            line_height: self.mix_gene(parent_a.line_height, parent_b.line_height),
            char_spacing: self.mix_gene(parent_a.char_spacing, parent_b.char_spacing),
            word_spacing: self.mix_gene(parent_a.word_spacing, parent_b.word_spacing),
        };

        self.mutate_enum_gene(&mut child.font, self.domains.fonts.len(), phase);
        self.mutate_enum_gene(&mut child.align, self.domains.aligns.len(), phase);
        self.mutate_enum_gene(&mut child.anchor, self.domains.anchors.len(), phase);
        self.mutate_numeric_gene(
            &mut child.translation_x,
            self.domains.translation_xs.len(),
            phase,
        );
        self.mutate_numeric_gene(
            &mut child.translation_y,
            self.domains.translation_ys.len(),
            phase,
        );
        self.mutate_numeric_gene(
            &mut child.world_scale_x,
            self.domains.world_scale_xs.len(),
            phase,
        );
        if self.domains.world_scale_bound {
            child.world_scale_y = child.world_scale_x;
        } else {
            self.mutate_numeric_gene(
                &mut child.world_scale_y,
                self.domains.world_scale_ys.len(),
                phase,
            );
        }
        self.mutate_numeric_gene(
            &mut child.line_height,
            self.domains.line_heights.len(),
            phase,
        );
        self.mutate_numeric_gene(
            &mut child.char_spacing,
            self.domains.char_spacings.len(),
            phase,
        );
        self.mutate_numeric_gene(
            &mut child.word_spacing,
            self.domains.word_spacings.len(),
            phase,
        );

        child
    }

    fn select_parent(&mut self, ranked: &[RankedGenome], phase: SearchPhase) -> CandidateGenome {
        let pool_size = ranked.len().min(self.parent_pool_size(phase)).max(1);
        let weight_sum = pool_size * (pool_size + 1) / 2;
        let mut ticket = self.rng.next_usize(weight_sum);
        for rank in 0..pool_size {
            let weight = pool_size - rank;
            if ticket < weight {
                return ranked[rank].genome;
            }
            ticket -= weight;
        }
        ranked[0].genome
    }

    fn mix_gene(&mut self, left: usize, right: usize) -> usize {
        if self.rng.next_f32() < 0.5 {
            left
        } else {
            right
        }
    }

    fn mutate_enum_gene(&mut self, gene: &mut usize, domain_len: usize, phase: SearchPhase) {
        if domain_len <= 1 || self.rng.next_f32() >= self.enum_mutation_rate(phase) {
            return;
        }
        *gene = self.rng.next_usize(domain_len);
    }

    fn mutate_numeric_gene(&mut self, gene: &mut usize, domain_len: usize, phase: SearchPhase) {
        if domain_len <= 1 {
            return;
        }
        if self.rng.next_f32() < self.numeric_reset_rate(phase) {
            *gene = self.rng.next_usize(domain_len);
            return;
        }
        if self.rng.next_f32() >= self.numeric_mutation_rate(phase) {
            return;
        }

        let radius = self.mutation_radius(domain_len, phase);
        let delta = self.rng.next_signed_offset(radius as i32);
        *gene = clamp_gene_index(*gene, delta, domain_len);
    }

    fn mutation_radius(&self, domain_len: usize, phase: SearchPhase) -> usize {
        let max_radius = (domain_len / 3).max(1);
        let anneal = 0.84_f32.powi(self.generation as i32);
        let phase_cap = match phase {
            SearchPhase::Explore => max_radius,
            SearchPhase::Focus => 12,
            SearchPhase::Refine => 4,
            SearchPhase::Polish => 1,
        };
        let stagnation_bonus = match self.generations_without_improvement {
            0..=1 => 0,
            2..=3 => 1,
            4..=6 => 2,
            _ => 4,
        };
        let radius = ((max_radius as f32) * anneal).round() as usize;
        let radius = radius.min(phase_cap).saturating_add(stagnation_bonus);
        radius.clamp(1, domain_len.saturating_sub(1).max(1))
    }

    fn current_phase(&self) -> SearchPhase {
        let score = self.best_fitness.max(0.0);
        let polish_threshold = (self.target_similarity - 0.01).clamp(0.89, 0.995);
        let refine_threshold = (self.target_similarity - 0.05).clamp(0.82, polish_threshold - 0.02);
        let focus_threshold = (self.target_similarity - 0.15).clamp(0.65, refine_threshold - 0.05);

        if score >= polish_threshold {
            SearchPhase::Polish
        } else if score >= refine_threshold {
            SearchPhase::Refine
        } else if score >= focus_threshold {
            SearchPhase::Focus
        } else {
            SearchPhase::Explore
        }
    }

    fn random_candidate_rate(&self, phase: SearchPhase) -> f32 {
        match phase {
            SearchPhase::Explore => 0.18,
            SearchPhase::Focus => 0.06,
            SearchPhase::Refine => {
                if self.generations_without_improvement >= 4 {
                    0.02
                } else {
                    0.0
                }
            }
            SearchPhase::Polish => {
                if self.generations_without_improvement >= 6 {
                    0.01
                } else {
                    0.0
                }
            }
        }
    }

    fn parent_pool_size(&self, phase: SearchPhase) -> usize {
        match phase {
            SearchPhase::Explore => self.population_size.clamp(4, 16),
            SearchPhase::Focus => self.population_size.clamp(4, 10),
            SearchPhase::Refine => self.elite_count.max(3),
            SearchPhase::Polish => self.elite_count.max(2),
        }
    }

    fn enum_mutation_rate(&self, phase: SearchPhase) -> f32 {
        match phase {
            SearchPhase::Explore => 0.12,
            SearchPhase::Focus => 0.05,
            SearchPhase::Refine => {
                if self.generations_without_improvement >= 3 {
                    0.01
                } else {
                    0.0
                }
            }
            SearchPhase::Polish => 0.0,
        }
    }

    fn numeric_reset_rate(&self, phase: SearchPhase) -> f32 {
        match phase {
            SearchPhase::Explore => 0.10,
            SearchPhase::Focus => 0.015,
            SearchPhase::Refine => {
                if self.generations_without_improvement >= 5 {
                    0.005
                } else {
                    0.0
                }
            }
            SearchPhase::Polish => 0.0,
        }
    }

    fn numeric_mutation_rate(&self, phase: SearchPhase) -> f32 {
        match phase {
            SearchPhase::Explore => 0.72,
            SearchPhase::Focus => 0.86,
            SearchPhase::Refine => 0.94,
            SearchPhase::Polish => 0.98,
        }
    }

    fn seed_neighbor_candidates(&self) -> Vec<CandidateGenome> {
        self.local_neighbors_from_radii(self.seed_genome, &[32, 16, 8, 4, 2, 1], true)
    }

    fn local_neighbor_candidates(
        &self,
        center: CandidateGenome,
        phase: SearchPhase,
    ) -> Vec<CandidateGenome> {
        match phase {
            SearchPhase::Explore => Vec::new(),
            SearchPhase::Focus => self.local_neighbors_from_radii(center, &[12, 6, 3, 1], false),
            SearchPhase::Refine => self.local_neighbors_from_radii(center, &[4, 2, 1], true),
            SearchPhase::Polish => self.local_neighbors_from_radii(center, &[2, 1], true),
        }
    }

    fn local_neighbors_from_radii(
        &self,
        center: CandidateGenome,
        radii: &[usize],
        include_pair_moves: bool,
    ) -> Vec<CandidateGenome> {
        let mut candidates = Vec::new();
        for &radius in radii {
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.translation_x,
                self.domains.translation_xs.len(),
                radius,
                |genome, next_index| genome.translation_x = next_index,
            );
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.translation_y,
                self.domains.translation_ys.len(),
                radius,
                |genome, next_index| genome.translation_y = next_index,
            );
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.world_scale_x,
                self.domains.world_scale_xs.len(),
                radius,
                |genome, next_index| genome.world_scale_x = next_index,
            );
            if !self.domains.world_scale_bound {
                self.push_index_neighbors(
                    &mut candidates,
                    center,
                    center.world_scale_y,
                    self.domains.world_scale_ys.len(),
                    radius,
                    |genome, next_index| genome.world_scale_y = next_index,
                );
            }
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.line_height,
                self.domains.line_heights.len(),
                radius,
                |genome, next_index| genome.line_height = next_index,
            );
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.char_spacing,
                self.domains.char_spacings.len(),
                radius,
                |genome, next_index| genome.char_spacing = next_index,
            );
            self.push_index_neighbors(
                &mut candidates,
                center,
                center.word_spacing,
                self.domains.word_spacings.len(),
                radius,
                |genome, next_index| genome.word_spacing = next_index,
            );

            if include_pair_moves {
                self.push_translation_pair_neighbors(&mut candidates, center, radius);
                self.push_scale_pair_neighbors(&mut candidates, center, radius);
            }
        }
        candidates
    }

    fn push_index_neighbors<F>(
        &self,
        candidates: &mut Vec<CandidateGenome>,
        center: CandidateGenome,
        current_index: usize,
        domain_len: usize,
        radius: usize,
        mut set_gene: F,
    ) where
        F: FnMut(&mut CandidateGenome, usize),
    {
        if domain_len <= 1 {
            return;
        }

        for delta in [-(radius as i32), radius as i32] {
            let next_index = clamp_gene_index(current_index, delta, domain_len);
            if next_index == current_index {
                continue;
            }
            let mut genome = center;
            set_gene(&mut genome, next_index);
            if self.domains.world_scale_bound {
                genome.world_scale_y = genome.world_scale_x;
            }
            candidates.push(genome);
        }
    }

    fn push_translation_pair_neighbors(
        &self,
        candidates: &mut Vec<CandidateGenome>,
        center: CandidateGenome,
        radius: usize,
    ) {
        if self.domains.translation_xs.len() <= 1 || self.domains.translation_ys.len() <= 1 {
            return;
        }

        for (dx, dy) in [
            (-(radius as i32), -(radius as i32)),
            (-(radius as i32), radius as i32),
            (radius as i32, -(radius as i32)),
            (radius as i32, radius as i32),
        ] {
            let next_x =
                clamp_gene_index(center.translation_x, dx, self.domains.translation_xs.len());
            let next_y =
                clamp_gene_index(center.translation_y, dy, self.domains.translation_ys.len());
            if next_x == center.translation_x && next_y == center.translation_y {
                continue;
            }
            let mut genome = center;
            genome.translation_x = next_x;
            genome.translation_y = next_y;
            candidates.push(genome);
        }
    }

    fn push_scale_pair_neighbors(
        &self,
        candidates: &mut Vec<CandidateGenome>,
        center: CandidateGenome,
        radius: usize,
    ) {
        if self.domains.world_scale_xs.len() <= 1 {
            return;
        }

        if self.domains.world_scale_bound {
            for delta in [-(radius as i32), radius as i32] {
                let next_scale = clamp_gene_index(
                    center.world_scale_x,
                    delta,
                    self.domains.world_scale_xs.len(),
                );
                if next_scale == center.world_scale_x {
                    continue;
                }
                let mut genome = center;
                genome.world_scale_x = next_scale;
                genome.world_scale_y = next_scale;
                candidates.push(genome);
            }
            return;
        }

        if self.domains.world_scale_ys.len() <= 1 {
            return;
        }

        for delta in [-(radius as i32), radius as i32] {
            let next_x = clamp_gene_index(
                center.world_scale_x,
                delta,
                self.domains.world_scale_xs.len(),
            );
            let next_y = clamp_gene_index(
                center.world_scale_y,
                delta,
                self.domains.world_scale_ys.len(),
            );
            if next_x == center.world_scale_x && next_y == center.world_scale_y {
                continue;
            }
            let mut genome = center;
            genome.world_scale_x = next_x;
            genome.world_scale_y = next_y;
            candidates.push(genome);
        }
    }
}

impl SimpleRng {
    fn from_entropy() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as u64)
            .unwrap_or(0x5eed_f00d_dead_beef);
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.state = state;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn next_usize(&mut self, upper_bound: usize) -> usize {
        if upper_bound <= 1 {
            0
        } else {
            (self.next_u64() % upper_bound as u64) as usize
        }
    }

    fn next_f32(&mut self) -> f32 {
        let value = self.next_u64() >> 40;
        (value as f32) / ((1u32 << 24) as f32)
    }

    fn next_signed_offset(&mut self, radius: i32) -> i32 {
        if radius <= 0 {
            return 0;
        }
        let width = (radius * 2 + 1) as usize;
        self.next_usize(width) as i32 - radius
    }
}

fn max_evaluation_budget(domains: &CandidateDomains) -> usize {
    let varying_dimensions = [
        domains.fonts.len(),
        domains.aligns.len(),
        domains.anchors.len(),
        domains.translation_xs.len(),
        domains.translation_ys.len(),
        domains.world_scale_xs.len(),
        if domains.world_scale_bound {
            1
        } else {
            domains.world_scale_ys.len()
        },
        domains.line_heights.len(),
        domains.char_spacings.len(),
        domains.word_spacings.len(),
    ]
    .into_iter()
    .filter(|len| *len > 1)
    .count();

    (varying_dimensions.max(1) * 48).clamp(64, 512)
}

fn default_population_size(domains: &CandidateDomains) -> usize {
    let varying_dimensions = [
        domains.fonts.len(),
        domains.aligns.len(),
        domains.anchors.len(),
        domains.translation_xs.len(),
        domains.translation_ys.len(),
        domains.world_scale_xs.len(),
        if domains.world_scale_bound {
            1
        } else {
            domains.world_scale_ys.len()
        },
        domains.line_heights.len(),
        domains.char_spacings.len(),
        domains.word_spacings.len(),
    ]
    .into_iter()
    .filter(|len| *len > 1)
    .count();

    (varying_dimensions.max(1) * 3).clamp(12, 28)
}

fn find_exact_index<T>(values: &[T], target: &T) -> usize {
    let target_discriminant = std::mem::discriminant(target);
    values
        .iter()
        .position(|value| std::mem::discriminant(value) == target_discriminant)
        .unwrap_or(0)
}

fn find_nearest_f32_index(values: &[f32], target: f32) -> usize {
    values
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            ((*left - target).abs()).total_cmp(&((*right - target).abs()))
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn clamp_gene_index(current: usize, delta: i32, domain_len: usize) -> usize {
    (current as i32 + delta).clamp(0, (domain_len - 1) as i32) as usize
}

fn resolve_bound_numeric_values(
    label: &str,
    left_mode: PropertyMode,
    left_values: &[f32],
    right_mode: PropertyMode,
    right_values: &[f32],
) -> Result<Vec<f32>> {
    match (
        matches!(left_mode, PropertyMode::Default),
        matches!(right_mode, PropertyMode::Default),
    ) {
        (true, true) => Ok(left_values.to_vec()),
        (false, true) => Ok(left_values.to_vec()),
        (true, false) => Ok(right_values.to_vec()),
        (false, false) => {
            if approx_f32_slices_equal(left_values, right_values) {
                Ok(left_values.to_vec())
            } else {
                bail!(
                    "`bindings.{label}` is `bound`, so x/y candidate lists must match; \
                     either keep one axis as `default` or give both axes the same values"
                )
            }
        }
    }
}

fn approx_f32_slices_equal(left_values: &[f32], right_values: &[f32]) -> bool {
    left_values.len() == right_values.len()
        && left_values
            .iter()
            .zip(right_values.iter())
            .all(|(left, right)| (left - right).abs() <= 0.0001)
}

#[derive(Debug, Clone, Copy)]
pub struct OptionalTextFieldDefaults {
    pub align_uses_default: bool,
    pub anchor_uses_default: bool,
    pub line_height_uses_default: bool,
    pub char_spacing_uses_default: bool,
    pub word_spacing_uses_default: bool,
}

pub fn build_view_layout(
    text: &str,
    parameters: &ConcreteTextParameters,
    optional_defaults: OptionalTextFieldDefaults,
) -> ViewLayoutAsset {
    ViewLayoutAsset {
        roots: vec![ViewNodeDef {
            name: "ReconstructedTextRoot".to_string(),
            tags: vec!["view_text_reconstruction".to_string()],
            style: Default::default(),
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: vec![TextDef {
                id: "ReconstructedText".to_string(),
                content: Some(text.to_string()),
                font: parameters.font.clone(),
                align: if optional_defaults.align_uses_default {
                    None
                } else {
                    Some(parameters.align)
                },
                anchor: if optional_defaults.anchor_uses_default {
                    None
                } else {
                    Some(parameters.anchor)
                },
                world_scale: (
                    Val::Static(parameters.world_scale_x),
                    Val::Static(parameters.world_scale_y),
                ),
                color: (
                    Val::Static(1.0),
                    Val::Static(1.0),
                    Val::Static(1.0),
                    Val::Static(1.0),
                ),
                transform: SerializableTransform {
                    translation: Some((
                        Val::Static(parameters.translation_x),
                        Val::Static(parameters.translation_y),
                        Val::Static(1.0),
                    )),
                    rotation: None,
                    scale: None,
                },
                line_height: if optional_defaults.line_height_uses_default {
                    None
                } else {
                    Some(parameters.line_height)
                },
                char_spacing: if optional_defaults.char_spacing_uses_default {
                    None
                } else {
                    Some(parameters.char_spacing)
                },
                word_spacing: if optional_defaults.word_spacing_uses_default {
                    None
                } else {
                    Some(parameters.word_spacing)
                },
                conditional_style: None,
                visible_when: None,
            }],
            view_box: None,
            children: Vec::new(),
            repeat: None,
        }],
        requires: Vec::new(),
        facts: None,
        world_space: false,
    }
}

pub fn parse_view_font(value: &str) -> Result<ViewFontDef> {
    let normalized = normalize_token(value);
    match normalized.as_str() {
        "determinationmono" | "dtmmono" => Ok(ViewFontDef::DeterminationMono),
        "determinationsans" | "dtmsans" => Ok(ViewFontDef::DeterminationSans),
        "hud" => Ok(ViewFontDef::Hud),
        "battlehud" => Ok(ViewFontDef::BattleHud),
        _ => bail!("unsupported font candidate `{value}`"),
    }
}

pub fn parse_text_align(value: &str) -> Result<TextAlignDef> {
    let normalized = normalize_token(value);
    match normalized.as_str() {
        "left" => Ok(TextAlignDef::Left),
        "center" => Ok(TextAlignDef::Center),
        "right" => Ok(TextAlignDef::Right),
        _ => bail!("unsupported align candidate `{value}`"),
    }
}

pub fn parse_text_anchor(value: &str) -> Result<TextAnchorDef> {
    let normalized = normalize_token(value);
    match normalized.as_str() {
        "topleft" => Ok(TextAnchorDef::TopLeft),
        "topcenter" => Ok(TextAnchorDef::TopCenter),
        "topright" => Ok(TextAnchorDef::TopRight),
        "centerleft" => Ok(TextAnchorDef::CenterLeft),
        "center" => Ok(TextAnchorDef::Center),
        "centerright" => Ok(TextAnchorDef::CenterRight),
        "bottomleft" => Ok(TextAnchorDef::BottomLeft),
        "bottomcenter" => Ok(TextAnchorDef::BottomCenter),
        "bottomright" => Ok(TextAnchorDef::BottomRight),
        _ => bail!("unsupported anchor candidate `{value}`"),
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|character| !matches!(character, '-' | '_' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}
