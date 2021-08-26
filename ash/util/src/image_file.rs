use ash::vk;

pub struct ImageFile {
    pub size: u64,
    pub width: u32,
    pub height: u32,
    pub data: std::vec::Vec<u8>
}

impl ImageFile {
    #[rustfmt::skip]
    pub fn new(path: &std::path::Path) -> ImageFile {
        use image::GenericImageView;

        let object = image::open(path).unwrap(); //.flipv();
        let (width, height) = (object.width(), object.height());
        let size = (std::mem::size_of::<u8>() as u32 * width * height * 4) as vk::DeviceSize;
        let data = match &object {
            image::DynamicImage::ImageLuma8(_) | 
            image::DynamicImage::ImageBgr8(_) | 
            image::DynamicImage::ImageRgb8(_) => {
                object.to_rgba().into_raw()
            }
            image::DynamicImage::ImageLumaA8(_) |
            image::DynamicImage::ImageBgra8(_) | 
            image::DynamicImage::ImageRgba8(_) => {
                object.raw_pixels()
            }
        };
        
        if size <= 0 {
            panic!("Failed to load texture image!")
        }

        ImageFile {
            size,
            width,
            height,
            data
        }
    }
}
