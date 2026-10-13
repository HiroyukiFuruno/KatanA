pub const PLUGIN_API_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExtensionPoint {
    RendererEnhancement,
    AiTool,
    UiPanel,
}

#[derive(Debug, Clone)]
pub struct PluginMeta {
    pub id: String,
    pub name: String,
    pub api_version: u32,
    pub extension_points: Vec<ExtensionPoint>,
}

#[derive(Debug)]
pub enum PluginInitResult {
    Ok,
    Failed(String),
    IncompatibleVersion { declared: u32, required: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Active,
    Disabled,
    IncompatibleVersion,
}

#[derive(Debug)]
pub struct PluginEntry {
    pub meta: PluginMeta,
    pub status: PluginStatus,
}

#[derive(Default)]
pub struct PluginRegistry {
    pub entries: Vec<PluginEntry>,
}
