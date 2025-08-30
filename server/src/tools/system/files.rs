pub struct FilesTool {
    user_id: uuid::Uuid,
    config: FilesToolConfig,
}

pub struct FilesToolConfig {}

impl FilesTool {
    pub fn new(user_id: uuid::Uuid, config: FilesToolConfig) -> Self {
        Self { user_id, config }
    }
}
