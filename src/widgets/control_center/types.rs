#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCenterSection {
    Volume,
    Mic,
    Bluetooth,
    Network,
    Monitor,
}

#[derive(Clone)]
pub struct ControlCenterStateChanged;
