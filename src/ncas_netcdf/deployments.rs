use super::netcdf_components::{Dimension, Variable};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Deployment {
    pub name: String,
    pub variables: Vec<Variable>,
    pub dimensions: Vec<Dimension>,
}

#[derive(Debug, Deserialize)]
pub struct DimensionCV {
    #[serde(rename = "Name", deserialize_with = "csv::invalid_option")]
    pub name: Option<String>,
    #[serde(rename = "Length", deserialize_with = "csv::invalid_option")]
    pub length: Option<u32>,
    #[serde(rename = "units", deserialize_with = "csv::invalid_option")]
    pub units: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VariableCV {
    #[serde(rename = "Variable", deserialize_with = "csv::invalid_option")]
    pub variable: Option<String>,
    #[serde(rename = "Attribute", deserialize_with = "csv::invalid_option")]
    pub attribute: Option<String>,
    #[serde(rename = "Value", deserialize_with = "csv::invalid_option")]
    pub value: Option<String>,
    #[serde(
        rename = "Compliance checking rules",
        deserialize_with = "csv::invalid_option"
    )]
    pub compliance: Option<String>,
}

async fn get_deployment_variables(
    deployment: &str,
    tag: &str,
) -> Result<Vec<Variable>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/_common/variables-"
        + deployment
        + ".tsv";
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
            //if parts.len() == 1 {
            if !parts[0].is_empty() {
                variable_name = parts[0].to_string();
            } else if !parts[2].is_empty() {
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

async fn get_deployment_dimensions(
    deployment: &str,
    tag: &str,
) -> Result<Vec<Dimension>, Box<dyn Error>> {
    let file_path = "https://raw.githubusercontent.com/ncasuk/AMF_CVs/".to_owned()
        + tag
        + "/product-definitions/tsv/_common/dimensions-"
        + deployment
        + ".tsv";
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
                    "Can not find name for dimension in deployment {}",
                    deployment
                )
                .into());
            }
        };
        dimensions.push(dimension);
    }
    Ok(dimensions)
}

pub async fn get_deployment(deployment: String, tag: String) -> Result<Deployment, Box<dyn Error>> {
    let variables = get_deployment_variables(&deployment, &tag).await?;
    let dimensions = get_deployment_dimensions(&deployment, &tag).await?;
    let deployment = Deployment {
        name: deployment,
        variables,
        dimensions,
    };
    Ok(deployment)
}
