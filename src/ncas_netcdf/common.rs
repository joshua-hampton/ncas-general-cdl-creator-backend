use super::netcdf_components::GlobalAttribute;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct CommonGlobalAttrsCV {
    #[serde(rename = "Name", deserialize_with = "csv::invalid_option")]
    pub name: Option<String>,
    #[serde(rename = "Description", deserialize_with = "csv::invalid_option")]
    pub description: Option<String>,
    #[serde(rename = "Example", deserialize_with = "csv::invalid_option")]
    pub example: Option<String>,
    #[serde(rename = "Fixed Value", deserialize_with = "csv::invalid_option")]
    pub fixed_value: Option<String>,
    #[serde(
        rename = "Compliance checking rules",
        deserialize_with = "csv::invalid_option"
    )]
    pub compliance_checking_rules: Option<String>,
    #[serde(
        rename = "Convention Providence",
        deserialize_with = "csv::invalid_option"
    )]
    pub convention_providence: Option<String>,
    #[serde(
        rename = "Regex check (if required) - AS & JS to populate Vocabulary",
        deserialize_with = "csv::invalid_option"
    )]
    pub regex_check: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Common {
    pub global_attributes: Vec<GlobalAttribute>,
}

async fn get_common_global_attrs(tag: &str) -> Result<Vec<GlobalAttribute>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/_common/global-attributes.tsv";
    let res = reqwest::get(&file_path).await?;
    let ga_data = res.text().await?;
    let mut lines = ga_data.lines();
    lines.next(); // Discard the first line
    let mut attrs: Vec<GlobalAttribute> = Vec::new();
    for line in lines {
        let parts = line.split('\t').collect::<Vec<&str>>();
        let attr = GlobalAttribute {
            name: parts.first().unwrap_or(&"").to_string(),
            value: parts.get(3).unwrap_or(&"").to_string(),
            example: parts.get(2).unwrap_or(&"").to_string(),
            compliance: parts.get(4).unwrap_or(&"").to_string(),
        };
        attrs.push(attr);
    }
    Ok(attrs)
}

pub async fn get_common(tag: String) -> Result<Common, Box<dyn Error>> {
    let global_attributes = get_common_global_attrs(&tag).await?;
    Ok(Common { global_attributes })
}
