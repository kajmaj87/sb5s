use crate::{Projection, ProjectionApi};

impl ProjectionApi {
    /// register new projection
    pub fn register_projection<P: Projection>(&self, projection: P) {
        self.projection_manager.register_projection(projection);
    }
}
