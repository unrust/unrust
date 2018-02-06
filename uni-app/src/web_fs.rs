use std;
use stdweb::web::TypedArray;
use std::cell::RefCell;
use std::rc::Rc;

pub type IoError = std::io::Error;
pub struct FileSystem {}
pub struct File {
    buffer: Rc<RefCell<Vec<u8>>>,
}

impl FileSystem {
    pub fn open(s: &str) -> Result<File, IoError> {
        let buffer = Rc::new(RefCell::new(vec![]));
        let buffer_p = buffer.clone();

        let get_buffer = move |ab: TypedArray<u8>| {
            *buffer_p.borrow_mut() = ab.to_vec();
        };

        js!{
            var oReq = new XMLHttpRequest();
            var filename = @{s};
            oReq.open("GET", filename, true);
            oReq.responseType = "arraybuffer";

            oReq.onload = function (oEvent) {
                var get_buffer = @{get_buffer};
                var arrayBuffer = oReq.response; // Note: not oReq.responseText
                if (arrayBuffer) {
                    get_buffer(new Uint8Array(arrayBuffer));
                }
                get_buffer.drop();
            };

            oReq.send(null);
        }

        Ok(File { buffer: buffer })
    }
}

impl File {
    pub fn is_ready(&self) -> bool {
        self.buffer.borrow().len() > 0
    }

    pub fn read_binary(&mut self) -> Result<Vec<u8>, IoError> {
        Ok(self.buffer.borrow().clone())
    }
}
