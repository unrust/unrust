use engine::core::{GameObject, SceneTree};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::instance;
use super::widgets;

use engine::IEngine;

type WidgetGoMap = HashMap<u32, (Rc<widgets::Widget>, Rc<RefCell<GameObject>>)>;

pub struct Context {
    go: WidgetGoMap,
    tree: Rc<SceneTree>,
}

impl Context {
    pub fn new(tree: Rc<SceneTree>) -> Context {
        Context {
            go: HashMap::new(),
            tree,
        }
    }

    pub fn reset(&mut self) {
        self.go.clear()
    }

    pub fn update(&mut self, inner: &instance::ImguiRaw, engine: &mut IEngine) {
        let (sw, sh) = engine.screen_size();

        for w in inner.render_list.iter() {
            let mut do_insert = {
                let hm = &self.go;
                match hm.get(&w.id()) {
                    None => true,
                    Some(&(ref oldw, _)) => **oldw != **w,
                }
            };

            if do_insert {
                let widget = match w.as_ref() {
                    &widgets::Widget::Label(ref label) => {
                        label.bind((sw, sh), &self.tree.root(), engine)
                    }
                    &widgets::Widget::Image(ref image) => {
                        image.bind((sw, sh), &self.tree.root(), engine)
                    }
                };

                self.go.insert(w.id(), (w.clone(), widget));
            }
        }

        // remove all go in hm which id >= last id
        self.go.retain(|k, _| *k <= inner.id);
    }
}
