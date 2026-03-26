use crate::config::PropertyTable;
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
    evaluation_budget: usize,
    population_size: usize,
    elite_count: usize,
    generation: usize,
    evaluations_done: usize,
    next_population_index: usize,
    population: Vec<CandidateGenome>,
    population_scores: Vec<Option<f32>>,
    last_issued_slot: Option<usize>,
    rng: SimpleRng,
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

impl CandidateSearchPlan {
    pub fn build(
        assume_single_line: bool,
        properties: &PropertyTable,
        defaults: ConcreteTextParameters,
    ) -> Result<Self> {
        let fonts = properties.font.resolve_font_values(defaults.font)?;
        let aligns = properties.align.resolve_align_values(defaults.align)?;
        let anchors = properties.anchor.resolve_anchor_values(defaults.anchor)?;
        let translation_xs = properties
            .translation_x
            .resolve_values("translation_x", defaults.translation_x)?;
        let translation_ys = properties
            .translation_y
            .resolve_values("translation_y", defaults.translation_y)?;
        let world_scale_xs = properties
            .world_scale_x
            .resolve_values("world_scale_x", defaults.world_scale_x)?;
        let world_scale_ys = properties
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
            world_scale_ys.len(),
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
            line_heights,
            char_spacings,
            word_spacings,
        };
        let evaluation_budget = full_search_space.min(max_evaluation_budget(&domains));
        let population_size = full_search_space
            .min(default_population_size(&domains))
            .max(1);
        let elite_count = population_size.clamp(1, 6);
        let rng = SimpleRng::from_entropy();

        Ok(Self {
            domains,
            full_search_space,
            evaluation_budget,
            population_size,
            elite_count,
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

    pub fn record_fitness(&mut self, fitness: f32) {
        let Some(slot) = self.last_issued_slot.take() else {
            return;
        };
        if let Some(score_slot) = self.population_scores.get_mut(slot) {
            *score_slot = Some(fitness);
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

    pub fn restart(&mut self) -> (usize, ConcreteTextParameters) {
        self.generation = 0;
        self.evaluations_done = 0;
        self.next_population_index = 0;
        self.population.clear();
        self.population_scores.clear();
        self.last_issued_slot = None;
        self.rng = SimpleRng::from_entropy();
        self.next_candidate()
            .expect("search plan always contains at least one candidate")
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
            world_scale_y: self.world_scale_ys[genome.world_scale_y],
            line_height: self.line_heights[genome.line_height],
            char_spacing: self.char_spacings[genome.char_spacing],
            word_spacing: self.word_spacings[genome.word_spacing],
        }
    }

    fn default_genome(&self, defaults: &ConcreteTextParameters) -> CandidateGenome {
        CandidateGenome {
            font: find_exact_index(&self.fonts, &defaults.font),
            align: find_exact_index(&self.aligns, &defaults.align),
            anchor: find_exact_index(&self.anchors, &defaults.anchor),
            translation_x: find_nearest_f32_index(&self.translation_xs, defaults.translation_x),
            translation_y: find_nearest_f32_index(&self.translation_ys, defaults.translation_y),
            world_scale_x: find_nearest_f32_index(&self.world_scale_xs, defaults.world_scale_x),
            world_scale_y: find_nearest_f32_index(&self.world_scale_ys, defaults.world_scale_y),
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

        self.generation += 1;
        self.next_population_index = 0;
        self.last_issued_slot = None;

        let remaining_budget = self.evaluation_budget.saturating_sub(self.evaluations_done);
        let target_population_size = self.population_size.min(remaining_budget).max(1);

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

        let mut attempts = 0usize;
        let max_attempts = target_population_size.saturating_mul(32).max(64);
        while next_population.len() < target_population_size
            && unique.len() < self.full_search_space
            && attempts < max_attempts
        {
            let candidate = if self.rng.next_f32() < 0.18 {
                self.random_genome()
            } else {
                self.breed_genome(&ranked)
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

        let remaining_budget = self.evaluation_budget.saturating_sub(self.evaluations_done);
        let target_population_size = self.population_size.min(remaining_budget).max(1);

        let mut population = Vec::with_capacity(target_population_size);
        let mut unique = HashSet::with_capacity(target_population_size);

        let default_genome = self
            .domains
            .default_genome(&ConcreteTextParameters::default());
        unique.insert(default_genome);
        population.push(default_genome);

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
        CandidateGenome {
            font: self.rng.next_usize(self.domains.fonts.len()),
            align: self.rng.next_usize(self.domains.aligns.len()),
            anchor: self.rng.next_usize(self.domains.anchors.len()),
            translation_x: self.rng.next_usize(self.domains.translation_xs.len()),
            translation_y: self.rng.next_usize(self.domains.translation_ys.len()),
            world_scale_x: self.rng.next_usize(self.domains.world_scale_xs.len()),
            world_scale_y: self.rng.next_usize(self.domains.world_scale_ys.len()),
            line_height: self.rng.next_usize(self.domains.line_heights.len()),
            char_spacing: self.rng.next_usize(self.domains.char_spacings.len()),
            word_spacing: self.rng.next_usize(self.domains.word_spacings.len()),
        }
    }

    fn breed_genome(&mut self, ranked: &[RankedGenome]) -> CandidateGenome {
        let parent_a = self.select_parent(ranked);
        let parent_b = self.select_parent(ranked);
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

        self.mutate_enum_gene(&mut child.font, self.domains.fonts.len());
        self.mutate_enum_gene(&mut child.align, self.domains.aligns.len());
        self.mutate_enum_gene(&mut child.anchor, self.domains.anchors.len());
        self.mutate_numeric_gene(&mut child.translation_x, self.domains.translation_xs.len());
        self.mutate_numeric_gene(&mut child.translation_y, self.domains.translation_ys.len());
        self.mutate_numeric_gene(&mut child.world_scale_x, self.domains.world_scale_xs.len());
        self.mutate_numeric_gene(&mut child.world_scale_y, self.domains.world_scale_ys.len());
        self.mutate_numeric_gene(&mut child.line_height, self.domains.line_heights.len());
        self.mutate_numeric_gene(&mut child.char_spacing, self.domains.char_spacings.len());
        self.mutate_numeric_gene(&mut child.word_spacing, self.domains.word_spacings.len());

        child
    }

    fn select_parent(&mut self, ranked: &[RankedGenome]) -> CandidateGenome {
        let pool_size = ranked.len().min(self.elite_count.max(1));
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

    fn mutate_enum_gene(&mut self, gene: &mut usize, domain_len: usize) {
        if domain_len <= 1 || self.rng.next_f32() >= 0.12 {
            return;
        }
        *gene = self.rng.next_usize(domain_len);
    }

    fn mutate_numeric_gene(&mut self, gene: &mut usize, domain_len: usize) {
        if domain_len <= 1 {
            return;
        }
        if self.rng.next_f32() < 0.12 {
            *gene = self.rng.next_usize(domain_len);
            return;
        }
        if self.rng.next_f32() >= 0.58 {
            return;
        }

        let radius = self.mutation_radius(domain_len);
        let delta = self.rng.next_signed_offset(radius as i32);
        *gene = clamp_gene_index(*gene, delta, domain_len);
    }

    fn mutation_radius(&self, domain_len: usize) -> usize {
        let max_radius = (domain_len / 3).max(1);
        let anneal = 0.84_f32.powi(self.generation as i32);
        let radius = ((max_radius as f32) * anneal).round() as usize;
        radius.clamp(1, domain_len.saturating_sub(1).max(1))
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
        domains.world_scale_ys.len(),
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
        domains.world_scale_ys.len(),
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
