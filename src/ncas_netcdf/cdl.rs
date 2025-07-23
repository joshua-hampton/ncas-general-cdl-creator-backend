use super::common::Common;
use super::data_products::DataProduct;
use super::deployments::Deployment;
use super::instruments::InstRecord;
use super::netcdf_components::{GlobalAttribute, Variable};
use std::error::Error;

#[derive(Debug)]
pub enum RequirementLevel {
    Required,
    RequiredIf(String),
    Optional,
}

const MAX_LINE_LENGTH: usize = 100;

fn requirement_level_string(level: &RequirementLevel, spaces: usize) -> String {
    let spaces_str = " ".repeat(spaces);
    match level {
        RequirementLevel::Required => format!("{}// Required", spaces_str),
        RequirementLevel::RequiredIf(reason) => format!("{}// Required if {}", spaces_str, reason),
        RequirementLevel::Optional => format!("{}// Optional", spaces_str),
    }
}

fn map_data_type(data_type: &str) -> String {
    match data_type {
        "int8" => "byte".to_string(),
        "byte" => "byte".to_string(),
        "int32" => "int".to_string(),
        "float32" => "float".to_string(),
        "float64" => "double".to_string(),
        "string" => "string".to_string(),
        _ => "unknown".to_string(),
    }
}

fn create_dimension_text(
    dimension_name: &str,
    dimension_length: Option<u32>,
    requirement_info: bool,
) -> String {
    let dim_length = match dimension_length {
        Some(length) => length.to_string(),
        None => "<dim length>".to_string(),
    };
    let dimension_text = format!("        {} = {} ;", dimension_name, dim_length);
    let len_dimension_text = dimension_text.len();
    let spaces = if len_dimension_text > MAX_LINE_LENGTH {
        2
    } else {
        MAX_LINE_LENGTH - len_dimension_text
    };
    let requirement_text = if requirement_info {
        requirement_level_string(&RequirementLevel::Required, spaces)
    } else {
        "".to_string()
    };
    format!("{}{}\n", dimension_text, requirement_text)
}

fn dimension_section(
    deployment: &Deployment,
    data_product: &DataProduct,
    requirement_info: bool,
) -> String {
    let mut section = String::new();
    section.push_str("dimensions:\n");
    for dimension in &deployment.dimensions {
        section.push_str(&create_dimension_text(
            &dimension.name,
            dimension.length,
            requirement_info,
        ));
    }
    for dimension in &data_product.dimensions {
        section.push_str(&create_dimension_text(
            &dimension.name,
            dimension.length,
            requirement_info,
        ));
    }
    section
}

fn create_variable_text(
    variable: &Variable,
    requirement_info: bool,
    requirement_level_variable: RequirementLevel,
    requirement_level_varattr: RequirementLevel,
) -> String {
    let default_type = &"unknown".to_string();
    let vartype = variable.attributes.get("type").unwrap_or(default_type);
    let vartype = &map_data_type(vartype);
    let vardims = variable
        .attributes
        .get("dimension")
        .map_or_else(|| "".to_string(), |dims| dims.to_string());
    let mut variable_text = format!("        {} {}({}) ;", vartype, variable.name, vardims);
    let len_variable_text = variable_text.len();
    let spaces = if len_variable_text > MAX_LINE_LENGTH {
        2
    } else {
        MAX_LINE_LENGTH - len_variable_text
    };
    let requirement_text = if requirement_info {
        requirement_level_string(&requirement_level_variable, spaces)
    } else {
        "".to_string()
    };
    variable_text.push_str(&requirement_text);
    variable_text.push('\n');
    for (attr_name, attr_value) in &variable.attributes {
        if attr_name == "type" || attr_name == "dimension" {
            continue; // Skip type and dimension attributes
        }
        let attr_text = if attr_name == "_FillValue" {
            format!(
                "                {}:{} = -1.e+20f ;",
                variable.name, attr_name
            )
        } else {
            format!(
                "                {}:{} = \"{}\" ;",
                variable.name, attr_name, attr_value
            )
        };
        let len_attr_text = attr_text.len();
        let spaces = if len_attr_text > MAX_LINE_LENGTH {
            2
        } else {
            MAX_LINE_LENGTH - len_attr_text
        };
        let requirement_text = if requirement_info {
            requirement_level_string(&requirement_level_varattr, spaces)
        } else {
            "".to_string()
        };
        variable_text.push_str(&format!("{}{}\n", attr_text, requirement_text));
    }
    variable_text
}

fn variable_section(
    deployment: &Deployment,
    data_product: &DataProduct,
    requirement_info: bool,
) -> String {
    let mut section = String::new();
    section.push_str("variables:\n");
    for variable in &deployment.variables {
        section.push_str(&create_variable_text(
            variable,
            requirement_info,
            RequirementLevel::Required,
            RequirementLevel::Required,
        ));
    }
    for variable in &data_product.variables {
        section.push_str(&create_variable_text(
            variable,
            requirement_info,
            RequirementLevel::Optional,
            RequirementLevel::RequiredIf("the variable is present".to_string()),
        ));
    }
    section
}

fn create_attribute_text(
    attr: &GlobalAttribute,
    instrument_record: &InstRecord,
    requirement_info: bool,
) -> String {
    let value = if attr.name == "instrument_manufacturer" {
        instrument_record.manufacturer.clone()
    } else if attr.name == "instrument_model" {
        instrument_record.model_no.clone()
    } else if attr.name == "instrument_serial_number" {
        instrument_record.serial_number.clone()
    } else if attr.name == "source" {
        instrument_record.descriptor.clone()
    } else {
        Some(if attr.value.is_empty() {
            format!("EXAMPLE: {}", attr.example.clone())
        } else {
            attr.value.clone()
        })
    };
    let value = match value {
        Some(v) => v,
        None => "EXAMPLE".to_string(),
    };
    let attr_text = format!("                {} = \"{}\" ;", attr.name, value);
    let len_attr_text = attr_text.len();
    let spaces = if len_attr_text > MAX_LINE_LENGTH {
        2
    } else {
        MAX_LINE_LENGTH - len_attr_text
    };
    let requirement_text = if requirement_info {
        requirement_level_string(&RequirementLevel::Required, spaces)
    } else {
        "".to_string()
    };
    format!("{}{}\n", attr_text, requirement_text)
}

fn attribute_section(
    common: &Common,
    data_product: &DataProduct,
    instrument_record: &InstRecord,
    requirement_info: bool,
) -> String {
    let mut section = String::new();
    section.push_str("\n// global attributes:\n");
    for attr in &common.global_attributes {
        section.push_str(&create_attribute_text(
            attr,
            instrument_record,
            requirement_info,
        ));
    }
    for attr in &data_product.global_attributes {
        section.push_str(&create_attribute_text(
            attr,
            instrument_record,
            requirement_info,
        ));
    }
    section
}

pub fn make_cdl(
    file_name: String,
    common: Common,
    deployment: Deployment,
    data_product: DataProduct,
    instrument_record: InstRecord,
    requirement_info: bool,
) -> Result<String, Box<dyn Error>> {
    let mut cdl = String::new();
    cdl.push_str(
        format!(
            "netcdf {} {{\n",
            file_name.strip_suffix(".nc").unwrap_or(&file_name)
        )
        .as_str(),
    );
    cdl.push_str(&dimension_section(
        &deployment,
        &data_product,
        requirement_info,
    ));
    cdl.push_str(&variable_section(
        &deployment,
        &data_product,
        requirement_info,
    ));
    cdl.push_str(&attribute_section(
        &common,
        &data_product,
        &instrument_record,
        requirement_info,
    ));
    cdl.push('}');
    Ok(cdl)
}
