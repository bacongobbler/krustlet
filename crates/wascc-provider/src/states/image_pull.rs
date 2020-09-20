use kubelet::state::prelude::*;
use log::error;

use crate::PodState;

use super::image_pull_backoff::ImagePullBackoff;
use super::volume_mount::VolumeMount;

/// Kubelet is pulling container images.
#[derive(Default, Debug)]
pub struct ImagePull;

#[async_trait::async_trait]
impl State<PodState> for ImagePull {
    async fn next(
        self: Box<Self>,
        pod_state: &mut PodState,
        pod: &Pod,
    ) -> anyhow::Result<Transition<PodState>> {
        let auth_resolver =
            kubelet::secret::RegistryAuthResolver::new(pod_state.shared.client.clone(), &pod);
        pod_state.run_context.modules = match pod_state
            .shared
            .store
            .fetch_pod_modules(&pod, &auth_resolver)
            .await
        {
            Ok(modules) => modules,
            Err(e) => {
                error!("{:?}", e);
                return Ok(Transition::next(self, ImagePullBackoff));
            }
        };
        Ok(Transition::next(self, VolumeMount))
    }

    async fn json_status(
        &self,
        _pod_state: &mut PodState,
        _pod: &Pod,
    ) -> anyhow::Result<serde_json::Value> {
        make_status(Phase::Pending, "ImagePull")
    }
}

impl EdgeTo<ImagePullBackoff> for ImagePull {}
impl EdgeTo<VolumeMount> for ImagePull {}