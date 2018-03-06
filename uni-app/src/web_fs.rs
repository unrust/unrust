use std;
use stdweb::web::TypedArray;
use std::cell::RefCell;
use std::rc::Rc;
use std::io::ErrorKind;

pub type IoError = std::io::Error;

pub struct FileSystem {}

enum BufferState {
    Empty,
    Buffer(Vec<u8>),
    Error(String),
}

pub struct File {
    buffer_state: Rc<RefCell<BufferState>>,
}

impl FileSystem {
    pub fn open(s: &str) -> Result<File, IoError> {
        let buffer_state = Rc::new(RefCell::new(BufferState::Empty));

        let on_get_buffer = {
            let buffer_state = buffer_state.clone();
            move |ab: TypedArray<u8>| {
                let data = ab.to_vec();
                if data.len() > 0 {
                    *buffer_state.borrow_mut() = BufferState::Buffer(data);
                }
            }
        };

        let on_error = {
            let buffer_state = buffer_state.clone();
            move |s: String| {
                let msg = format!("Fail to read file from web {}", s);
                *buffer_state.borrow_mut() = BufferState::Error(msg);
            }
        };

        js!{
            var oReq = new XMLHttpRequest();
            var filename = @{s};
            oReq.open("GET", filename, true);
            oReq.responseType = "arraybuffer";

            var on_error_js = function(s){
                var on_error = @{on_error};
                on_error(s);
                on_error.drop();
            };

            oReq.onload = function (oEvent) {
                var status = oReq.status;
                var arrayBuffer = oReq.response; // Note: not oReq.responseText
                if (status == 200 && arrayBuffer) {
                    var on_get_buffer = @{on_get_buffer};
                    on_get_buffer(new Uint8Array(arrayBuffer));
                    on_get_buffer.drop();
                } else {
                    on_error_js("Fail to get array buffer from network..");
                }
            };

            oReq.onerror = function(oEvent) {
                on_error_js("Fail to read from network..");
            };

            oReq.send(null);
        }

        Ok(File {
            buffer_state: buffer_state,
        })
    }
}

impl File {
    pub fn is_ready(&self) -> bool {
        let bs = self.buffer_state.borrow();
        match *bs {
            BufferState::Empty => false,
            BufferState::Error(_) => true,
            BufferState::Buffer(_) => true,
        }
    }

    pub fn read_binary(&mut self) -> Result<Vec<u8>, IoError> {
        let mut bs = self.buffer_state.borrow_mut();
        match *bs {
            BufferState::Error(ref s) => Err(std::io::Error::new(ErrorKind::Other, s.clone())),
            BufferState::Buffer(ref mut v) => Ok({
                let mut r = Vec::new();
                r.append(v);
                r
            }),
            _ => unreachable!(),
        }
    }
}
