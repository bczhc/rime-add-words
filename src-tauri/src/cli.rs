use serde::Serialize;

#[derive(clap::Parser, Debug, Serialize, Clone)]
#[command(author, version, about)]
#[serde(rename_all = "camelCase")]
pub struct AppArgs {
    #[arg()]
    pub dict_path: Option<String>,
    #[arg()]
    pub char_map_path: Option<String>,
}
