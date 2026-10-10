use std::io::Write;

static FONT: &[u8] = include_bytes!("../Inter/Inter_18pt-Black.ttf");

pub fn makepdf(name: &str, keytxt: &str, w: impl Write) {
    let data = genpdf::fonts::FontData::new(FONT.to_vec(), None).expect("failed to load font");
    let font_family = genpdf::fonts::FontFamily {
        regular: data.clone(),
        bold: data.clone(),
        italic: data.clone(),
        bold_italic: data.clone(),
    };
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(name);
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    doc.push(genpdf::elements::Paragraph::new(name));
    doc.push(genpdf::elements::Paragraph::new(keytxt));

    let code = qrcode::QrCode::new(keytxt).expect("failed to create a qr code from the key");
    let image = code.render::<image::Luma<u8>>().
        min_dimensions(1000, 1000).
        build();
    let dyn_img = image::DynamicImage::ImageLuma8(image);
    let pdf_img = genpdf::elements::Image::from_dynamic_image(dyn_img).expect("failed to get image in pdf format");
    doc.push(pdf_img);

    doc.render(w).expect("failed to render the pdf");
}