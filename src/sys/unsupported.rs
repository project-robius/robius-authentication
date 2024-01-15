use crate::BiometricStrength;

#[derive(Debug)]
pub(crate) struct PolicyBuilder;

impl PolicyBuilder {
    pub(crate) const fn new() -> Self {
        Self
    }

    pub(crate) const fn biometrics(self, _: Option<BiometricStrength>) -> Self {
        Self
    }

    pub(crate) const fn password(self, _: bool) -> Self {
        Self
    }

    pub(crate) const fn watch(self, _: bool) -> Self {
        Self
    }

    // pub(crate) const fn wrist_detection(self, _: bool) -> Self {
    //  Self
    // }

    pub(crate) const fn build(self) -> Option<Policy> {
        None
    }
}

pub(crate) struct Policy;

pub(crate) struct Context;

impl Context {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn authenticate(&self, _: &Policy) -> bool {
        unimplemented!()
    }

    pub(crate) fn blocking_authenticate(&self, _: &Policy) -> bool {
        unimplemented!()
    }
}
