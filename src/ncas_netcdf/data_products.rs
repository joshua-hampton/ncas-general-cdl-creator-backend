use super::netcdf_components::{Dimension, GlobalAttribute, Variable};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct DataProduct {
    pub name: String,
    pub variables: Vec<Variable>,
    pub dimensions: Vec<Dimension>,
    pub global_attributes: Vec<GlobalAttribute>,
}

#[derive(Debug, Deserialize)]
pub struct DimensionCV {
    #[serde(rename = "Name", deserialize_with = "csv::invalid_option")]
    pub name: Option<String>,
    #[serde(rename = "Length", deserialize_with = "csv::invalid_option")]
    pub length: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct GlobalAttributeCV {
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
}

#[derive(Debug, Deserialize)]
pub struct VariableCV {
    #[serde(rename = "Variable", deserialize_with = "csv::invalid_option")]
    pub variable: Option<String>,
    #[serde(rename = "attribute", deserialize_with = "csv::invalid_option")]
    pub attribute: Option<String>,
    #[serde(rename = "Value", deserialize_with = "csv::invalid_option")]
    pub value: Option<String>,
    #[serde(rename = "Proposed name", deserialize_with = "csv::invalid_option")]
    pub proposed_name: Option<String>,
    #[serde(rename = "example value", deserialize_with = "csv::invalid_option")]
    pub example_value: Option<String>,
}

async fn get_data_product_global_attributes(
    data_product: &str,
    tag: &str,
) -> Result<Vec<GlobalAttribute>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/"
        + data_product
        + "/global-attributes-specific.tsv";
    let res = reqwest::get(&file_path).await?;
    let ga_data = res.text().await?;
    let mut attrs: Vec<GlobalAttribute> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(ga_data.as_bytes());
    for result in rdr.deserialize() {
        let record: GlobalAttributeCV = result?;
        let attr = match record.name {
            Some(name) => GlobalAttribute {
                name,
                value: record.fixed_value.unwrap_or_default(),
                example: record.example.unwrap_or_default(),
                compliance: record.compliance_checking_rules.unwrap_or_default(),
            },
            None => {
                return Err(format!(
                    "Can not find name for global attribute in data product {}",
                    data_product
                )
                .into());
            }
        };
        attrs.push(attr);
    }
    Ok(attrs)
}

async fn get_data_product_variables(
    data_product: &str,
    tag: &str,
) -> Result<Vec<Variable>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/"
        + data_product
        + "/variables-specific.tsv";
    let res = reqwest::get(&file_path).await?;
    let v_data = res.text().await?;
    // Skip the first line (header)
    let mut lines = v_data.lines();
    lines.next(); // Discard the first line

    // Re-join the remaining lines to re-split into blocks
    let response_body = lines.collect::<Vec<_>>().join("\n");
    let blocks = response_body.split("\n\n");

    let mut variables = Vec::new();

    for block in blocks {
        let lines = block.lines();
        let mut variable_name = String::new();
        let mut attributes = HashMap::new();

        for line in lines {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.is_empty() {
                continue;
            }
            if parts.len() == 1 {
                variable_name = parts[0].to_string();
            } else if parts.len() == 3 {
                let attr_name = parts[1].to_string();
                let attr_value = parts[2].to_string();
                // Reformat flag values
                let attr_value = if attr_name == "flag_values" {
                    reformat_flag_values(&attr_value)
                } else if attr_name == "flag_meanings" {
                    reformat_flag_meanings(&attr_value)
                } else {
                    attr_value
                };
                attributes.insert(attr_name, attr_value);
            }
        }

        if !variable_name.is_empty() {
            variables.push(Variable {
                name: variable_name,
                attributes,
            });
        }
    }

    Ok(variables)
}

fn reformat_flag_values(flag_values: &str) -> String {
    // convert e.g. "0b, 1b, 2b,3b 4b" to "0, 1, 2, 3, 4"
    let mut flag_values = flag_values.replace("b", "");
    flag_values = flag_values.replace(" ", "");
    flag_values = flag_values.replace(",", ", ");
    flag_values
}

fn reformat_flag_meanings(flag_meanings: &str) -> String {
    // convert stringwith | separator to space separator
    // e.g. "meaning_1|meaning_2| meaning_3" to "meaning_1 meaning_2 meaning_3"
    let mut flag_meanings = flag_meanings.replace("|", " ");
    flag_meanings = flag_meanings.replace("  ", " ");
    flag_meanings
}

async fn get_data_product_dimensions(
    data_product: &str,
    tag: &str,
) -> Result<Vec<Dimension>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/"
        + data_product
        + "/dimensions-specific.tsv";
    let res = reqwest::get(&file_path).await?;
    let dp_data = res.text().await?;
    let mut dimensions: Vec<Dimension> = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(dp_data.as_bytes());
    for result in rdr.deserialize() {
        let record: DimensionCV = result?;
        let dimension = match record.name {
            Some(name) => Dimension {
                name,
                length: record.length,
            },
            None => {
                return Err(format!(
                    "Can not find name for dimension in data product {}",
                    data_product
                )
                .into());
            }
        };
        dimensions.push(dimension);
    }
    Ok(dimensions)
}

pub async fn get_data_product(
    data_product: String,
    tag: String,
) -> Result<DataProduct, Box<dyn Error>> {
    let variables = get_data_product_variables(&data_product, &tag).await?;
    let dimensions = get_data_product_dimensions(&data_product, &tag).await?;
    let global_attributes = get_data_product_global_attributes(&data_product, &tag).await?;
    let data_product = DataProduct {
        name: data_product,
        variables,
        dimensions,
        global_attributes,
    };
    Ok(data_product)
}
