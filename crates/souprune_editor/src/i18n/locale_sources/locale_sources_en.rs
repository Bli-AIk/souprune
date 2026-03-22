pub const EN_FTL: &str = "\
# Panel titles
panel-sequence-timeline = Sequence Timeline
panel-chapter-inspector = Chapter Inspector
panel-asset-browser = Asset Browser
panel-game-preview = Game Preview
panel-playback = Playback Control
panel-fre = FRE Panel
panel-view-editor = View Editor

# Chapter type names
chapter-spawn-view = Spawn View
chapter-await-fact = Await Fact
chapter-set-view-fact = Set View Fact
chapter-danmaku-performance = Danmaku Performance
chapter-am-performance = AM Performance
chapter-tween-view-element = Tween View Element
chapter-wait = Wait
chapter-sequence = Sequence
chapter-parallel = Parallel
chapter-set-player = Set Player
chapter-set-ui = Set UI
chapter-modify-view-element = Modify View Element
chapter-set-camera = Set Camera
chapter-conditional = Conditional
chapter-fact-switch = Fact Switch
chapter-emit-fact-event = Emit Fact Event
chapter-modify-fact = Modify Fact
chapter-load-fre = Load FRE
chapter-run-sequence = Run Sequence
chapter-load-map = Load Map
chapter-set-bgm = Set BGM
chapter-custom = Custom

# Actions
action-add = Add
action-delete = Delete
action-copy = Copy
action-paste = Paste
action-undo = Undo
action-redo = Redo
action-save = Save
action-open = Open
action-refresh = Refresh
action-create = Create
action-cancel = Cancel
action-find-refs = Find References
action-play-from-here = Play From Here
action-add-subcondition = + Add Subcondition
action-add-modification = + Add Modification
action-add-item = + Add

# Playback
playback-play = ▶ Play
playback-pause = ⏸ Pause
playback-stop = ⏹ Stop
playback-resume = ▶ Resume
playback-step = ⏭ Step
playback-chapter-progress = Chapter {$processed}/{$total}
playback-mode-edit = Edit
playback-mode-playing = ▶ Playing
playback-mode-paused = ⏸ Paused

# Common labels
label-needs-world = Requires World access
label-no-sequence = Open a .sequence.ron file to begin editing.
label-selected-chapter = Selected chapter: #{$index}
label-chapters = {$count} chapters
label-modified = ● Modified
label-unsaved = ● Unsaved
label-no-file-open = No file open
label-no-view-open = No View file open. Double-click a .view.ron in Asset Browser.
label-no-fre-open = No FRE file open
label-preview-not-init = Preview not initialized
label-no-data = No data
label-select-node = Select a node to edit properties
label-node-path-invalid = Node path invalid
label-parse-error = Parse error: {$err}
label-not-initialized = Not initialized
label-no-sequence-open = No sequence open
label-select-chapter = Select a chapter to view properties
label-invalid-chapter = Invalid chapter index
label-chapter-count = {$count} chapters
label-sub-chapters = Sub-chapters: {$count}
label-branch-count = Branches: {$count}
label-param-count = Parameters: {$count}
label-modification-count = Modifications: {$count}
label-no-project = Project directory not found
label-crossref-todo = (Cross-reference will be available in a future version)
label-find-refs-for = Finding references for '{$path}'...
label-no-simulated-facts = No simulated facts (FRE file has no initial facts)
label-no-facts = (no facts)
label-empty = (empty)
label-read-error = Failed to read file: {$err}
label-save-error = Save failed: {$err}
label-count-suffix = {$label}: {$count}

# Property labels
prop-name = Name
prop-tags = Tags
prop-condition = Condition
prop-fact-key = Fact Key
prop-event-id = Event ID
prop-action-type = Action Type
prop-variants = Variants
prop-key = Key
prop-value = Value
prop-branch = Branch {$index}
prop-duration-sec = Duration (sec)
prop-view-layout-file = View layout file
prop-bindings = Bindings
prop-perf-file = Performance file
prop-position = Position
prop-amproj-file = AMPROJ file
prop-am-config = AM Config
prop-data = Data
prop-fre-files = FRE Files
prop-seq-path = Sequence path
prop-dynamic-path-fact = Dynamic path Fact
prop-map-path = Map path
prop-bgm-path = BGM path
prop-fade-in-sec = Fade in (sec)
prop-params = Parameters
prop-texture-path = Texture path
prop-visibility = Visibility
prop-width = Width
prop-height = Height
prop-scale-x = ScaleX
prop-scale-y = ScaleY
prop-scale-z = ScaleZ
prop-modes = Modes
prop-config-path = Config path
prop-duration = Duration
prop-intensity = Intensity
prop-path-id = Path/ID
prop-element-id = Element ID
prop-content = Content
prop-variable-name = Variable name
prop-anim-clip = Animation clip

# Tree context menu
tree-add-child = Add Child Node
tree-move-up = Move Up
tree-move-down = Move Down

# View editor
view-node-tree = Node Tree
view-add-root = Add root node
view-properties = Properties
view-basics = Basics
view-width = Width:
view-height = Height:
view-data-requirements = Data Requirements
view-initial-facts = Initial Facts
view-repeat = Repeat
view-color = Color
view-font = Font

# View preview
preview-play = Play
preview-stop = Stop
preview-reset = Reset
preview-zoom = Zoom: {$percent}%
preview-input-active = Input Active

# FRE panel
fre-filter = Filter:
fre-db-unavailable = LayeredFactDatabase not available.
fre-global-layer = Global Layer
fre-local-layer = Local Layer
fre-add-fact = Add New Fact
fre-key = Key:
fre-value = Value:
fre-type = Type:
fre-layer = Layer:
fre-registry-unavailable = LayeredRuleRegistry not available.
fre-rules-total = Total: {$total} (Global: {$global}, Local: {$local})
fre-no-rules = No rules registered.
fre-global-rules = Global Rules ({$count})
fre-local-rules = Local Rules ({$count})
fre-trigger = Trigger:
fre-event-tracking-not-init = Event tracking not initialized.
fre-recent-events = Recent events: {$count}
fre-no-events = No events recorded yet.
fre-current = Current:
fre-state-config-not-loaded = StateConfig not loaded
fre-rules-count = FRE Rules ({$count})
fre-fact-simulator = Fact Simulator
fre-fact-simulator-live = Fact Simulator (Live)
fre-facts-count = Facts ({$count})
fre-priority = Priority: {$value}
fre-conditions = Conditions:
fre-actions = Actions:
fre-modifications = Modifications:
fre-outputs = Outputs: {$value}
fre-tabs-facts = Facts
fre-tabs-rules = Rules
fre-tabs-events = Events
fre-tabs-states = States

# Asset browser
browser-new-sequence = New Sequence
browser-new-view = New View
browser-new-rule = New Rule
browser-new-folder = New Folder
browser-refresh-tree = Refresh file tree
browser-search-hint = Search...
browser-new-file = New File
browser-name = Name:
browser-directory = Directory: {$path}

# Chapter inspector
inspector-use-local-facts = Use local Facts
inspector-wait-completion = Wait for completion
inspector-default-branch = Default branch
inspector-generate-collision = Generate collision
inspector-process-objects = Process objects
inspector-setup-camera-bounds = Setup camera bounds
inspector-selector = Selector:
inspector-modify-type = Modify type: {$label}
inspector-specify-position = Specify position
inspector-position = Position:
inspector-active = Active
inspector-follow-player = Follow player

# Sequence timeline
timeline-open-sequence = Open sequence file
timeline-save = Save

# Widgets
widget-browse-file = Browse file
widget-static = Static
widget-expression = Expression
widget-subconditions = Sub-conditions: {$count}

# Undo descriptions
undo-insert-chapter = Insert chapter
undo-remove-chapter = Remove chapter
undo-move-chapter = Move chapter
undo-modify-chapter = Modify chapter

# Chapter palette categories
palette-flow = Flow Control
palette-scene = Scene
palette-view = View
palette-logic = Logic
palette-combat = Combat
palette-audio = Audio
palette-extension = Extension

# Asset browser categories
browser-cat-sequence = Sequence
browser-cat-view = View
browser-cat-rule = Rule
browser-cat-performance = Performance
browser-cat-config = Config
browser-cat-other = Other
browser-cat-directory = Directory

# Preview
label-preview-init = Preview (initializing...)

# File picker
picker-all-files = All files

# Zoom prefix
prop-zoom-prefix = Zoom: 
";
