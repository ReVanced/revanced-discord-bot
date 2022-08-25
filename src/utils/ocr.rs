use reqwest::Error;
use tesseract::Tesseract;

pub async fn get_text_from_image_url(url: &str) -> Result<String, Error> {
    let image = &reqwest::get(url).await?.bytes().await.unwrap().to_vec();
    Ok(Tesseract::new(None, None)
        .unwrap()
        .set_image_from_mem(image)
        .unwrap()
        .get_text()
        .unwrap())
}
