use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct InstRecord {
    #[serde(rename = "Instrument", deserialize_with = "csv::invalid_option")]
    pub instrument: Option<String>,
    #[serde(rename = "Manufacturer", deserialize_with = "csv::invalid_option")]
    pub manufacturer: Option<String>,
    #[serde(rename = "Model No.", deserialize_with = "csv::invalid_option")]
    pub model_no: Option<String>,
    #[serde(rename = "Serial Number", deserialize_with = "csv::invalid_option")]
    pub serial_number: Option<String>,
    #[serde(
        rename = "Old Instrument Name",
        deserialize_with = "csv::invalid_option"
    )]
    pub old_instrument_name: Option<String>,
    #[serde(
        rename = "New Instrument Name",
        deserialize_with = "csv::invalid_option"
    )]
    pub instrument_name: Option<String>,
    #[serde(rename = "Data Product(s)", deserialize_with = "csv::invalid_option")]
    pub data_product: Option<String>,
    #[serde(
        rename = "Mobile/Fixed (loc)",
        deserialize_with = "csv::invalid_option"
    )]
    pub mobile_fixed: Option<String>,
    #[serde(rename = "Host", deserialize_with = "csv::invalid_option")]
    pub host: Option<String>,
    #[serde(rename = "Scientist", deserialize_with = "csv::invalid_option")]
    pub scientist: Option<String>,
    #[serde(rename = "Category", deserialize_with = "csv::invalid_option")]
    pub category: Option<String>,
    #[serde(rename = "Descriptor", deserialize_with = "csv::invalid_option")]
    pub descriptor: Option<String>,
    #[serde(rename = "Owner", deserialize_with = "csv::invalid_option")]
    pub owner: Option<String>,
    #[serde(rename = "PID", deserialize_with = "csv::invalid_option")]
    pub pid: Option<String>,
}

pub async fn get_instrument_data(instrument_name: String) -> Result<InstRecord, Box<dyn Error>> {
    let instrument_name = Some(instrument_name);
    let file_path = "https://raw.githubusercontent.com/ncasuk/ncas-data-instrument-vocabs/refs/heads/main/product-definitions/tsv/_instrument_vocabs/ncas-instrument-name-and-descriptors.tsv";
    let mut instrument_record = InstRecord {
        instrument: None,
        manufacturer: None,
        model_no: None,
        serial_number: None,
        old_instrument_name: None,
        instrument_name: instrument_name.clone(),
        data_product: None,
        mobile_fixed: None,
        host: None,
        scientist: None,
        category: None,
        descriptor: None,
        owner: None,
        pid: None,
    };
    let res = reqwest::get(file_path).await?;
    let inst_data = res.text().await?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(inst_data.as_bytes());
    for result in rdr.deserialize() {
        let record: InstRecord = result?;
        if record.instrument_name == instrument_name {
            instrument_record = record;
        }
    }
    Ok(instrument_record)
}
