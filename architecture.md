# Bevy Granite Architecture

## Table of Contents
1. [Overview](#overview)
2. [Project Structure](#project-structure)
3. [Core Crates](#core-crates)
4. [Data Flow](#data-flow)
5. [Key Components](#key-components)
6. [Entity System](#entity-system)
7. [Serialization & Deserialization](#serialization--deserialization)
8. [Editor Architecture](#editor-architecture)
9. [Gizmo System](#gizmo-system)
10. [Plugin System](#plugin-system)

---

## Overview

**Bevy Granite** is an interactive 3D editor framework built on top of the Bevy game engine. It provides a complete solution for:

- **Scene Creation & Editing**: Visually create and edit 3D scenes
- **Entity Management**: Create, modify, delete, and organize entities
- **Serialization**: Save and load scenes with complete state preservation
- **Component Extension**: Add custom components via procedural macros
- **Gizmo-based Manipulation**: Transform entities using 3D gizmos
- **Undo/Redo System**: Full command history with edit reversal

### Key Features

- **Modular Architecture**: Optional features (core, gizmos, editor)
- **Macro-based Registration**: Automatic component registration via `#[granite_component]`
- **Flexible Serialization**: Support for full disk preservation, transform-only, or runtime-only saving
- **UI Framework**: Egui-based editor interface with dockable panels
- **Event-driven**: Comprehensive event system for editor control

---

## Project Structure

```bash
bevy_granite/
├── crates/
│   ├── bevy_granite_core/          # Core serialization, entity handling, asset management
│   ├── bevy_granite_editor/        # Editor UI, input handling, viewport rendering
│   ├── bevy_granite_gizmos/        # Transform gizmos, selection, picking
│   ├── bevy_granite_logging/       # Structured logging system
│   ├── bevy_granite_macros/        # Procedural macros for component registration
│   ├── bevy_granite_expose/        # Bevy internal component exposure
│   └── bevy_granite_gizmos/        # (Alternative gizmos implementation)
├── assets/
│   ├── scenes/                     # Example scene files
│   ├── textures/                   # Default textures
│   └── models/                     # OBJ models
├── examples/
│   └── dungeon/                    # Complete example implementation
└── docs/
    └── architecture.md             # This file
```

---

## Core Crates

### 1. **bevy_granite_core**

The foundational crate containing serialization, deserialization, and entity management.

#### Key Modules

**`src/entities/`**

- **`mod.rs`**: Core entity types and traits
  - `IdentityData`: UUID, name, and class information
  - `TransformData`: Position, rotation, scale
  - `MainCamera`, `UICamera`: Marker components for special cameras
  - `SpawnSource`: Tracks entity origin and save settings
  - `SaveSettings`: Controls serialization behavior (Runtime, PreserveDiskTransform, PreserveDiskFull)

- **`editable/`**: Editable entity type system
  - `definition.rs`: Entity type definitions and metadata
  - `category.rs`: Entity categorization (lights, cameras, models, etc.)
  - `types/`: Built-in entity types
    - `camera_3d/`: 3D camera with FOV, near/far planes
    - `directional_light/`: Sun-like light sources
    - `point_light/`: Omnidirectional lights
    - `spot_light/`: Cone-shaped lights
    - `obj/`: OBJ model loading and rendering
    - `rect_brush/`: Rectangular brush for level design
    - `empty/`: Empty transform nodes
    - Each type has: `mod.rs` (definition), `creation.rs` (factory), `plugin.rs` (Bevy setup), `ui.rs` (editor UI), `update_event.rs` (change events)

- **`serialize.rs`**: Entity serialization pipeline
  - `SceneMetadata`: Format version and entity count
  - `SceneData`: Complete scene structure
  - `EntitySaveReadyData`: Individual entity save format
  - `serialize_entities()`: Main serialization function with save setting support

- **`deserialize.rs`**: Entity deserialization and world loading
  - Rebuilds entities from serialized data
  - Handles parent-child relationships
  - Applies reflected components

- **`component_editor.rs`**: Runtime component inspection/editing
  - Provides UI for custom components
  - Supports component addition/removal

- **`lifecycle.rs`**: Entity creation/destruction events
  - Handles entity spawning during deserialization
  - Cleanup on despawn

- **`generate_tangents.rs`**: Tangent generation for normal mapping

**`src/world/`**

- **`mod.rs`**: World state management
  - `WorldState`: Container for world data during save/load
  - `SaveWorldRequestData`: Pending save operations

- **`save.rs`**: Save pipeline
  - `save_request_system()`: Collects entity data and runtime components
  - Handles save settings (Runtime, PreserveDiskTransform, PreserveDiskFull)
  - Triggers component collection via events

- **`open.rs`**: World loading
  - Deserializes scenes from disk
  - Rebuilds entity hierarchies
  - Applies transforms and components

- **`reload.rs`**: Scene reloading with state preservation

**`src/assets/`**

- **`materials/`**: Material management
  - `definition.rs`: Material type definitions
  - `load.rs`: Material loading from files
  - `AvailableEditableMaterials`: Registry of available materials

- **`plugin.rs`**: Asset system plugin

**`src/shared/`**

- **`file.rs`**: File system utilities
- **`file_browser.rs`**: File selection dialogs
- **`icon.rs`**: Icon loading and caching
- **`user_input.rs`**: Input event handling (keyboard, mouse)
- **`version.rs`**: Scene format versioning

**`src/events.rs`**
Core events for world management:

- `RequestSaveEvent`: Save current world
- `RequestLoadEvent`: Load world from path
- `RequestReloadEvent`: Reload from disk
- `RequestDespawnSerializableEntities`: Clear scene
- `RequestDespawnBySource`: Remove entities from specific source
- `CollectRuntimeDataEvent`: Gather runtime component data
- `RuntimeDataReadyEvent`: Notification when collection complete
- `WorldLoadSuccessEvent`: Save/load completion notifications

### 2. **bevy_granite_editor**

Complete editor UI and viewport implementation.

#### Key Modules

**`src/interface/`**

- **`plugin.rs`**: Main interface orchestration
- **`layout.rs`**: Dockable panel layout
- **`panels/`**: Editor windows
  - `SidePanel`: Entity tree, properties
  - `BottomPanel`: Logs, debug info, events
- **`tabs/`**: Individual editor tabs
  - `NodeTreeTabData`: Entity hierarchy view
  - `EditorSettingsTabData`: Editor configuration
  - `LogTabData`: Real-time logging output
  - `DebugTabData`: Debug information
- **`events.rs`**: Editor control events
  - `RequestEditorToggle`: Show/hide editor UI
  - `RequestSelectEntityEvent`: Select entity
  - `RequestDeselectEntityEvent`: Deselect entity
  - `RequestDeselectAllEntitiesEvent`: Clear selection
  - `RequestCameraEntityFrame`: Focus camera on entity
  - `RequestNewParent`: Reparent entities
  - `RequestRemoveParents`: Unparent entities
  - `RequestToggleCameraSync`: Sync editor camera with game camera
- **`cache.rs`**: Entity data caching for performance
- **`popups/`**: Modal dialogs (save, load, create)
- **`themes/`**: UI theming and styling

**`src/input/`**

- Keyboard and mouse input handling
- Camera control (orbit, pan, zoom)
- Selection interaction

**`src/viewport/`**

- **`ViewPortCamera`**: Marker for editor viewport camera
- Render target management
- Camera synchronization

**`src/history/`**

- **`CommandHistory`**: Undo/redo system
- **`EditorCommand`**: Base command trait
  - `EntityCreateCommand`: Create entity
  - `EntityDeleteCommand`: Delete entity
  - `TransformCommand`: Modify transform
- **`CommandResult`/`CommandError`**: Command execution results

**`src/editor_state/`**

- **`ConfigPlugin`**: Editor configuration management
- **`UI_CONFIG`**: UI layout and behavior settings
- **`HELP_CONFIG`**: Help text and tooltips

**`src/entities/`**

- Asset and entity browsing
- Entity bounds calculation
- Model/texture preview

**`src/setup.rs`**

- UI style initialization
- Editor startup configuration

### 3. **bevy_granite_gizmos**

3D transformation gizmos and selection system.

#### Key Modules

**`src/gizmos/`**

- **`TransformGizmo`**: Combined translate/rotate/scale gizmo
  - **`GizmoType`**: Translation, rotation, scaling modes
  - **`GizmoAxis`**: X, Y, Z, or plane-based movement
  - **`GizmoSnap`**: Grid snapping configuration
  - **`RotateGizmo`**: Individual rotation gizmo component
  - **`GizmoChildren`**: Child mesh entities for gizmo display
  - **`GizmoMesh`**: Mesh resource handles

- **`vertex/`**: Vertex-level picking visualization

**`src/selection/`**

- **`Selected`**: Marker for selected entities
- **`ActiveSelection`**: Resource tracking current selection
- **`SelectionPlugin`**: Selection event handling
- **`EntityEvents`**: Entity-specific events

**`src/input/`**

- **`DragState`**: Tracks ongoing gizmo interactions
- **`watch_gizmo_change()`**: System monitoring gizmo modifications
- Mouse drag handling for gizmo movement

**`src/camera/`**

- **`GizmoCamera`**: Dedicated gizmo rendering camera
- **`MainCameraAdded`**: Event when main camera is detected
- Automatic camera synchronization

**`src/ui/`**

- Gizmo mode selection UI
- Snap configuration UI
- Gizmo visibility toggling

### 4. **bevy_granite_macros**

Procedural macros for component and event registration.

#### Macros

**`#[granite_component]`**

```rust
#[granite_component(default)]
pub struct MyComponent {
    pub value: f32,
}

```

- Derives: `Reflect`, `Serialize`, `Deserialize`, `Component`, `Clone`, `Debug`, `Default`, `PartialEq`
- Adds reflection metadata for Bevy component inspection
- Registers with component editor
- Optional `ui_hidden` attribute for editor visibility

**`#[register_editor_components]`**
Macro that automatically registers all `#[granite_component]` decorated types with the Bevy app:

```rust
register_editor_components!(app);

```

- Registers type with Bevy's type registry
- Adds `BridgeTag` data for non-hidden components
- Enables UI visibility for editable components

**`#[ui_callable_events]`**
Registers event struct fields as clickable UI buttons:

```rust
#[ui_callable_events]
pub struct DebugEvents {
    pub spawn_player: DebugRequestPlayer,
    pub remove_player: DebugRequestRemovePlayer,
}

```

- Generates event sender closures
- Registers with editor UI system
- Creates clickable button for each field

### 5. **bevy_granite_logging**

Structured logging system with colored output and file logging.

#### Features

- **Log Levels**: Debug, Info, Warning, Error
- **Log Categories**: System, Input, Asset, Entity, etc.
- **Log Types**: Editor, Engine, Asset
- **File Output**: Automatic log file creation
- **Colored Console**: Terminal output with color coding
- **Timestamp**: Per-message timestamps

### 6. **bevy_granite_expose**

Bridges Bevy internal types with the editor's component system, allowing inspection and editing of built-in Bevy components.

---

## Data Flow

### Save Pipeline

```bash
User clicks "Save"
    ↓
RequestSaveEvent triggered
    ↓
save_request_system():
  1. Queries all entities with IdentityData + Transform
  2. Filters by SpawnSource (scene path)
  3. Collects entity data (id, transform, parent)
  4. Sends CollectRuntimeDataEvent
    ↓
Component collectors gather:
  - Registered components via type registry
  - Component data as serialized strings
    ↓
RuntimeDataReadyEvent
    ↓
serialize_entities():
  1. Combines entity data with components
  2. Applies SaveSettings logic:
     - Runtime: Use current world state
     - PreserveDiskTransform: Keep original transform
     - PreserveDiskFull: Keep all original data
  3. Serializes to RON format
  4. Writes to disk
    ↓
WorldSaveSuccessEvent
```

### Load Pipeline

```bash
User opens scene file / RequestLoadEvent
    ↓
world/open.rs:
  1. Reads scene file (RON)
  2. Parses metadata and entity list
  3. Creates temporary Entity → UUID map
    ↓
For each entity:
  1. Spawn base entity with Transform
  2. Add IdentityData (uuid, name, class)
  3. Add SpawnSource (tracking origin)
  4. Apply parent relationships via UUID mapping
  5. Instantiate class-specific bundle (Camera, Light, Model, etc.)
  6. Deserialize and add custom components
    ↓
WorldLoadSuccessEvent
```

### Entity Modification Pipeline

```bash
User interacts with gizmo / UI property
    ↓
gizmo/input.rs or component_editor.rs processes change
    ↓
[If Editor active]:
  CommandHistory records change:
    - EntityDeleteCommand / EntityCreateCommand
    - TransformCommand (position, rotation, scale)
    ↓
[If Gizmos active]:
  Transform updated via DragState
  GizmoChildren updated for visual feedback
    ↓
Entity component updated in Bevy world
```

---

## Key Components

### Entity Type System

Entities are defined by their "class" (GraniteType), which determines their bundle composition:

```bash
IdentityData (always present)
  ├── uuid: Uuid
  ├── name: String
  └── class: GraniteTypes (Camera3D, DirLight, PointLight, OBJ, etc.)

TransformData (always present)
  ├── translation: Vec3
  ├── rotation: Quat
  └── scale: Vec3

Optional Parent Entity

Optional Custom Components
  └── HashMap<ComponentTypeName, SerializedData>
```

### Entity Classes

Each class encapsulates creation, UI, and update logic:

| Class | Module | Use Case |
|-------|--------|----------|
| `Camera3D` | `camera_3d/` | Perspective/orthographic camera |
| `DirLight` | `directional_light/` | Sun-like directional light |
| `PointLight` | `point_light/` | Omnidirectional point light |
| `SpotLight` | `spot_light/` | Cone-shaped spotlight |
| `OBJ` | `obj/` | 3D model rendering |
| `RectBrush` | `rect_brush/` | Rectangular brush primitive |
| `Empty` | `empty/` | Transform node without rendering |
| `Unknown` | `unknown.rs` | Fallback for unrecognized types |

### Save Settings

Control how entities are preserved during save/load:

- **Runtime**: Uses current world state at save time
- **PreserveDiskTransform**: Keeps original transform, updates all other data
- **PreserveDiskFull**: Preserves complete original data from disk

---

## Serialization & Deserialization

### RON Format Example

```ron
(
  metadata: (
    format_version: (major: 0, minor: 3),
    entity_count: 3,
  ),
  entities: [
    (
      identity: (
        uuid: "550e8400-e29b-41d4-a716-446655440000",
        name: "Main Camera",
        class: Camera3D((
          fov: 1.5708,
          near: 0.1,
          far: 1000.0,
        )),
      ),
      transform: (
        translation: (0.0, 5.0, 10.0),
        rotation: (x: 0.0, y: 0.0, z: 0.0, w: 1.0),
        scale: (1.0, 1.0, 1.0),
      ),
      parent: None,
      components: None,
    ),
  ],
)
```

### Component Serialization

Custom components registered with `#[granite_component]` are automatically:

1. Reflected via Bevy's reflection system
2. Serialized to JSON strings during save
3. Deserialized and reconstructed during load
4. Validated by type registry

---

## Editor Architecture

### Plugin Initialization

```bash
BevyGraniteEditor::build()
    ├── FrameTimeDiagnosticsPlugin (Bevy FPS tracking)
    ├── CommandHistoryPlugin (Undo/Redo)
    ├── InputPlugin (Keyboard/Mouse)
    ├── InterfacePlugin (UI panels and events)
    ├── ViewportPlugin (Render target)
    ├── AssetPlugin (Material/texture loading)
    ├── ConfigPlugin (Editor config)
    └── BevyGraniteExposePlugin (Bevy type exposure)
```

### UI Architecture

Egui-based interface with docking:

```bash
┌─────────────────────────────────────┐
│           Main Menu                 │
├──────────────┬──────────────────────┤
│              │                      │
│  Side Panel  │   Viewport           │
│  (Tree/Prop) │   (3D View)          │
│              │                      │
├──────────────┴──────────────────────┤
│        Bottom Panel (Logs)           │
└─────────────────────────────────────┘
```

#### Dockable Panels

**Side Panel (SideDockState)**

- Node Tree Tab: Entity hierarchy visualization
- Properties Tab: Entity property editing
- Assets Tab: Asset browser

**Bottom Panel (BottomDockState)**

- Log Tab: Real-time log output
- Debug Tab: Debug information
- Events Tab: Callable event buttons

#### Property Editors

For each component type, an editor is generated:

- **Built-in Types**: Transform, Camera, Light properties
- **Custom Components**: Automatically generated from component definition
- **Field Types**: Vec3, Quat, f32, String, enum, bool, etc.

### Event System

The editor controls via events, allowing decoupled systems:

```bash
User Action → Event → System Handler → World Update
```

Example event flow:

```bash
Click entity in tree
  ↓
RequestSelectEntityEvent
  ↓
Selection system marks entity as Selected
  ↓
Gizmo renders at entity position
  ↓
Transform gizmo displayed in viewport
```

### Command History

Undo/Redo implementation using command pattern:

```bash
User action (transform, create, delete)
  ↓
EditorCommand trait implementation created
  ↓
Command.execute() modifies world
  ↓
Command stored in CommandHistory stack
  ↓
Undo: Pop, call Command.undo()
Redo: Pop from undo, call Command.execute()
```

---

## Gizmo System

### Transform Gizmo Architecture

```bash
TransformGizmo (Parent Entity)
├── GizmoChildren (contains mesh references)
├── DragState (tracks interaction)
├── GizmoType (Translation/Rotation/Scale mode)
├── GizmoSnap (snap settings)
└── GizmoAxis (active axis)

Visual Representation:
├── X-axis: Red arrow/handle
├── Y-axis: Green arrow/handle
├── Z-axis: Blue arrow/handle
└── Center: White center point
```

### Selection & Picking

```bash
MeshPickingPlugin (Raycasting)
    ↓
Picks mesh under cursor
    ↓
Selection system identifies entity
    ↓
Mark with Selected component
    ↓
Gizmo spawned at entity position
```

### Gizmo Interaction Flow

```bash
Mouse down on gizmo
  ↓
DragState activated with:
  - Current axis
  - Starting position
  - Starting transform
  ↓
Mouse move
  ↓
Compute delta from start
  ↓
Apply delta to transform based on mode:
  - Translation: Direct position offset
  - Rotation: Quaternion update
  - Scale: Vector magnitude update
  ↓
Mouse release
  ↓
DragState finalized
  ↓
[If Editor]: Record TransformCommand for undo
```

### Vertex Picking

Allows selection and visualization of individual mesh vertices for detailed editing.

---

## Plugin System

### Feature Flags

```toml
[features]
default = ["core", "editor", "gizmos"]
core = ["bevy_granite_core", "bevy_granite_logging", "bevy_granite_macros"]
editor = ["core", "gizmos", "bevy_granite_editor"]
gizmos = ["core", "bevy_granite_gizmos"]
```

### Plugin Dependencies

```bash
BevyGraniteCore
├── EntityPlugin
├── WorldPlugin
├── AssetPlugin
├── SharedPlugin
└── Events (save, load, despawn, etc.)

BevyGraniteGizmos
├── BevyGraniteCore (required)
├── MeshPickingPlugin
├── GizmoPlugin
├── SelectionPlugin
├── InputPlugin
└── UIPlugin

BevyGraniteEditor
├── BevyGraniteCore (required)
├── BevyGraniteGizmos (required)
├── FrameTimeDiagnosticsPlugin
├── CommandHistoryPlugin
├── InputPlugin
├── InterfacePlugin
├── ViewportPlugin
├── AssetPlugin
├── ConfigPlugin
└── BevyGraniteExposePlugin
```

### Plugin Integration Example

```rust
use bevy::prelude::*;
use bevy_granite::prelude::*;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins)
        
        // Granite plugins
        .add_plugins(BevyGraniteCore { logging: true })
        .add_plugins(BevyGraniteGizmos)
        .add_plugins(BevyGraniteEditor {
            active: true,
            default_world: "assets/scenes/default.ron".to_string(),
        })
        
        // Custom systems
        .add_systems(Update, my_game_logic)
        
        // Register custom components
        .register_editor_components!()
        
        .run();
}

#[granite_component(default)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

---

## Type Registry & Reflection

### Component Registration Flow

```bash
#[granite_component] macro
    ↓
Registers in REGISTERED_COMPONENTS (static)
    ↓
register_editor_components!() macro
    ↓
Calls app.register_type::<YourComponent>()
    ↓
Calls app.register_type_data::<YourComponent, BridgeTag>()
    ↓
Component available for:
  - Serialization/Deserialization
  - Editor UI generation
  - Runtime inspection
```

### Reflection System

Bevy's reflection allows:

- **Type Discovery**: Finding available components at runtime
- **Dynamic Access**: Getting/setting values without compile-time knowledge
- **Serialization**: Converting components to/from strings
- **UI Generation**: Automatically creating editors from field types

---

## Asset System

### Material Management

Materials are defined and can be:

1. Created via `NewEditableMaterial` in the editor
2. Loaded from files (TOML definitions)
3. Applied to OBJ entities
4. Cached for reuse

### Texture Loading

Supported formats (via Bevy):

- PNG
- JPEG
- TGA

Textures can be:

- Loaded individually
- Loaded with repeat settings
- Applied to materials
- Cached in memory

---

## Error Handling & Validation

### Version Compatibility

Scenes include format version:

```ron
format_version: (major: 0, minor: 3)
```

Validation checks:

- Major version match (breaking changes)
- Minor version compatibility (forward compatible)

### File I/O

- File browser for safe path selection
- Dialog-based save/open (native dialogs)
- Graceful error handling with logging
- Path normalization (absolute ↔ relative)

---

## Performance Considerations

### Caching

- **EntityUIDataCache**: Caches expensive entity queries
- **Icon Cache**: Pre-loads icon textures
- **Material Cache**: Prevents redundant loads

### Optimization Strategies

- **Lazy Component Collection**: Runtime data only collected on save
- **Selective Serialization**: Skip unchanged components
- **Viewport Culling**: Only render visible entities
- **Batched Rendering**: Efficient mesh rendering

### Profiling Integration

- FrameTimeDiagnosticsPlugin for FPS tracking
- Bevy's built-in profiling support
- Log timing for save/load operations

---

## Extension Points

### Adding Custom Entity Types

```rust
// 1. Define in editable/types/my_type/mod.rs
#[derive(Reflect, Serialize, Deserialize, Clone, Debug)]
pub struct MyType {
    pub custom_field: f32,
}

// 2. Creation logic in creation.rs
pub fn create_my_type(commands: &mut Commands, entity_data: &MyType) {
    // Spawn with MyType bundle
}

// 3. UI editor in ui.rs
pub fn my_type_ui(/* ... */) {
    // egui property controls
}

// 4. Register in editable/types/mod.rs
```

### Adding Custom Components

```rust
#[granite_component(default)]
pub struct MyComponent {
    pub value: f32,
}

// Register in app setup
register_editor_components!();
```

### Custom Events

```rust
#[derive(Event, Default)]
pub struct MyCustomEvent;

#[ui_callable_events]
pub struct MyEvents {
    pub my_event: MyCustomEvent,
}

// Register UI
MyEvents::register_ui();
```

---

## Debugging

### Logging System

Enable logging in BevyGraniteCore:

```rust
.add_plugins(BevyGraniteCore { logging: true })
```

Log categories:

- System operations
- Input handling
- Asset loading
- Entity lifecycle
- Serialization/Deserialization

### Log Output

- **Console**: Colored, real-time output
- **File**: Timestamped log file in user home directory
- **Editor Panel**: In-app log viewer

---

## Summary

Bevy Granite provides a modular, event-driven architecture for building game editors on top of Bevy. The system is built on these core principles:

1. **Modularity**: Optional features via crates and feature flags
2. **Extensibility**: Macros for easy component/event registration
3. **Serialization**: Flexible scene saving with multiple strategies
4. **Visual Editing**: Gizmo-based 3D manipulation
5. **History**: Full undo/redo support
6. **Type Safety**: Bevy's reflection system for runtime type handling
7. **Event-Driven**: Decoupled systems communicating via events

The architecture supports both standalone editor usage and integration with existing Bevy applications.
