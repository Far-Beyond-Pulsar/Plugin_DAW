//! # DAW Editor Plugin
//!
//! This plugin provides a professional Digital Audio Workstation (DAW) for game audio.
//! It supports .pdaw files with real-time multi-track mixing, automation, and effects.
//!
//! ## File Types
//!
//! - **DAW Project** (.pdaw)
//!   - Contains complete DAW project with tracks, clips, and automation
//!   - JSON-based format for easy editing and version control
//!
//! ## Editors
//!
//! - **DAW Editor**: Full-featured DAW with timeline, mixer, and browser panels

use plugin_editor_api::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;
use gpui::*;
use ui::dock::PanelView;

// DAW Editor modules
mod daw_editor;

// Re-export main types
pub use daw_editor::DawEditorPanel;
pub use daw_editor::AudioService;

/// Storage for editor instances owned by the plugin
struct EditorStorage {
    panel: Arc<dyn ui::dock::PanelView>,
    wrapper: Box<DawEditorWrapper>,
}

/// The DAW Editor Plugin
pub struct DawEditorPlugin {
    editors: Arc<Mutex<HashMap<usize, EditorStorage>>>,
    next_editor_id: Arc<Mutex<usize>>,
}

impl Default for DawEditorPlugin {
    fn default() -> Self {
        Self {
            editors: Arc::new(Mutex::new(HashMap::new())),
            next_editor_id: Arc::new(Mutex::new(0)),
        }
    }
}

impl EditorPlugin for DawEditorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: PluginId::new("com.pulsar.daw-editor"),
            name: "DAW Editor".into(),
            version: "0.1.0".into(),
            author: "Pulsar Team".into(),
            description: "Professional Digital Audio Workstation for game audio design".into(),
        }
    }

    fn file_types(&self) -> Vec<FileTypeDefinition> {
        vec![
            FileTypeDefinition {
                id: FileTypeId::new("daw_project"),
                extension: "pdaw".to_string(),
                display_name: "DAW Project".to_string(),
                icon: ui::IconName::MusicNote,
                color: gpui::rgb(0x9C27B0).into(),
                structure: FileStructure::Standalone,
                default_content: json!({
                    "version": 1,
                    "name": "New Project",
                    "created_at": "",
                    "modified_at": "",
                    "sample_rate": 48000.0,
                    "tracks": [],
                    "transport": {
                        "tempo": 120.0,
                        "time_signature": [4, 4],
                        "loop_enabled": false,
                        "loop_start": 0,
                        "loop_end": 0,
                        "metronome_enabled": false
                    },
                    "master_track": {
                        "id": 0,
                        "name": "Master",
                        "track_type": "Master",
                        "volume": 0.8,
                        "pan": 0.0,
                        "muted": false,
                        "solo": false,
                        "armed": false,
                        "color": [0.5, 0.5, 0.5],
                        "clips": [],
                        "automation": [],
                        "sends": []
                    }
                }),
                categories: vec!["Audio".to_string()],
            }
        ]
    }

    fn editors(&self) -> Vec<EditorMetadata> {
        vec![EditorMetadata {
            id: EditorId::new("daw-editor"),
            display_name: "DAW Editor".into(),
            supported_file_types: vec![FileTypeId::new("daw_project")],
        }]
    }

    fn create_editor(
        &self,
        editor_id: EditorId,
        file_path: PathBuf,
        window: &mut Window,
        cx: &mut App,
        logger: &plugin_editor_api::EditorLogger,
    ) -> Result<(Arc<dyn PanelView>, Box<dyn EditorInstance>), PluginError> {
        logger.info("DAW EDITOR LOADED!!");
        if editor_id.as_str() == "daw-editor" {
            let panel = cx.new(|cx| DawEditorPanel::new_with_project(file_path.clone(), window, cx));
            let panel_arc: Arc<dyn ui::dock::PanelView> = Arc::new(panel.clone());
            let wrapper = Box::new(DawEditorWrapper {
                panel: panel.into(),
                file_path: file_path.clone(),
            });

            let id = {
                let mut next_id = self.next_editor_id.lock().unwrap();
                let id = *next_id;
                *next_id += 1;
                id
            };

            self.editors.lock().unwrap().insert(id, EditorStorage {
                panel: panel_arc.clone(),
                wrapper: wrapper.clone(),
            });

            log::info!("Created DAW editor instance {} for {:?}", id, file_path);
            Ok((panel_arc, wrapper))
        } else {
            Err(PluginError::EditorNotFound { editor_id })
        }
    }

    fn on_load(&mut self) {
        log::info!("DAW Editor Plugin loaded");
    }

    fn on_unload(&mut self) {
        let mut editors = self.editors.lock().unwrap();
        let count = editors.len();
        editors.clear();
        log::info!("DAW Editor Plugin unloaded (cleaned up {} editors)", count);
    }
}

#[derive(Clone)]
pub struct DawEditorWrapper {
    panel: Entity<DawEditorPanel>,
    file_path: std::path::PathBuf,
}

impl plugin_editor_api::EditorInstance for DawEditorWrapper {
    fn file_path(&self) -> &std::path::PathBuf {
        &self.file_path
    }

    fn save(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_save(window, cx)
        })
    }

    fn reload(&mut self, window: &mut Window, cx: &mut App) -> Result<(), PluginError> {
        self.panel.update(cx, |panel, cx| {
            panel.plugin_reload(window, cx)
        })
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

export_plugin!(DawEditorPlugin);
