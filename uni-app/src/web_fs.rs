use std;
use stdweb::web::TypedArray;
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::unstable::TryInto;

pub type IoError = std::io::Error;
pub struct FileSystem {}
pub struct File {
    buffer: Rc<RefCell<Vec<u8>>>,
    filename: String,
}

impl FileSystem {
    pub fn open(s: &str) -> Result<File, IoError> {
        let buffer = Rc::new(RefCell::new(vec![]));

        js!{
            var oReq = new XMLHttpRequest();
            var filename = @{s};
            oReq.open("GET", filename, true);
            oReq.responseType = "arraybuffer";
            if (Module.files == null) {
                Module.files = {};
            }

            Module.files[filename] = new Uint8Array(0);

            oReq.onload = function (oEvent) {
                var arrayBuffer = oReq.response; // Note: not oReq.responseText
                if (arrayBuffer) {
                    Module.files[filename] = new Uint8Array(arrayBuffer);
                }
            };

            oReq.send(null);
        }

        Ok(File {
            buffer: buffer,
            filename: String::from(s),
        })
    }
}

impl File {
    pub fn is_ready(&self) -> bool {
        let mut buffer = self.buffer.borrow_mut();
        if buffer.len() > 0 {
            return true;
        }

        let buffer_js = js!{
            return Module.files[@{&self.filename}];
        };

        let arr: TypedArray<u8> = buffer_js.try_into().unwrap();
        *buffer = arr.to_vec();
        buffer.len() > 0
    }

    pub fn read_binary(&mut self) -> Result<Vec<u8>, IoError> {
        Ok(self.buffer.borrow().clone())
    }
}
