use super::widgets;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Default, Debug, PartialEq, Copy, Clone)]
pub struct ImguiState {
    pub pivot: super::Metric,
    pub text_align: super::TextAlign,
}

#[derive(Default, Debug)]
pub struct ImguiRaw {
    pub id: u32,
    pub state: ImguiState,
    pub render_list: Vec<Rc<widgets::Widget>>,
}

pub struct Imgui {
    pub inner: Arc<Mutex<ImguiRaw>>,
}

thread_local!(
    static INSTANCE: Arc<Mutex<ImguiRaw>> = Arc::new(Mutex::new(Default::default()))
);

pub fn imgui_inst() -> Imgui {
    return Imgui {
        inner: INSTANCE.with(|f| f.clone()),
    };
}
