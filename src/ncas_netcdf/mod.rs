#![allow(dead_code, unused_variables)]

mod cdl;
mod common;
mod data_products;
mod deployments;
mod instruments;
mod netcdf_components;
use std::error::Error;

pub struct CDLData {
    pub filename: String,
    pub cdl: String,
}

pub async fn main(
    instrument_name: String,
    data_product: String,
    deployment: String,
    start_date: String,
    tag: String,
    include_requirement_info: bool,
) -> Result<CDLData, Box<dyn Error>> {
    let common = match common::get_common(tag.clone()).await {
        Ok(common) => common,
        Err(err) => {
            return Err(err);
        }
    };
    let instrument_record = match instruments::get_instrument_data(instrument_name.clone()).await {
        Ok(instrument_record) => instrument_record,
        Err(err) => {
            return Err(err);
        }
    };
    let data_product =
        match data_products::get_data_product(data_product.clone(), tag.clone()).await {
            Ok(data_product) => data_product,
            Err(err) => {
                return Err(err);
            }
        };
    let deployment = match deployments::get_deployment(deployment.clone(), tag.clone()).await {
        Ok(deployment) => deployment,
        Err(err) => {
            return Err(err);
        }
    };
    let mut platform = instrument_record
        .mobile_fixed
        .clone()
        .unwrap_or("fixed - unknown".to_string());
    platform = if platform.contains("fixed - ") {
        platform
            .split(" - ")
            .nth(1)
            .unwrap_or("unknown")
            .to_lowercase()
    } else {
        platform.to_lowercase()
    };
    let file_name = format!(
        "{}_{}_{}_{}_v1.0.nc",
        instrument_record
            .instrument_name
            .clone()
            .unwrap_or("unknown".to_string()),
        platform,
        start_date,
        data_product.name
    );
    let cdl = match cdl::make_cdl(
        file_name.clone(),
        common,
        deployment,
        data_product,
        instrument_record,
        include_requirement_info,
    ) {
        Ok(cdl) => cdl,
        Err(err) => {
            return Err(err);
        }
    };
    let cdl_data = CDLData {
        filename: file_name,
        cdl,
    };
    Ok(cdl_data)
}
