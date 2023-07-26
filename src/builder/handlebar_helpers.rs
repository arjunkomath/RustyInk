use chrono::{LocalResult, TimeZone, Utc};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    ScopedJson,
};
use serde_json::{json, value::Value as Json, Map};

#[derive(Clone, Copy)]
pub struct SliceHelper;

impl HelperDef for SliceHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let array = h
            .param(0)
            .map(|v| v.value())
            .ok_or(RenderError::new("Object/Array not found"))?;
        let start = h
            .param(1)
            .and_then(|v| v.value().as_u64())
            .ok_or(RenderError::new("Start index not found"))?;
        let end = h
            .param(2)
            .and_then(|v| v.value().as_u64())
            .ok_or(RenderError::new("End index not found"))?;

        match array {
            Json::Array(array) => {
                let new_array = array
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i >= start as usize && i <= end as usize)
                    .map(|(_, value)| value.clone())
                    .collect::<Vec<Json>>();

                return Ok(ScopedJson::Derived(Json::Array(new_array)));
            }
            Json::Object(object) => {
                let new_object = object
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i >= start as usize && i <= end as usize)
                    .map(|(_, (key, value))| (key.clone(), value.clone()))
                    .collect::<serde_json::Map<String, Json>>();

                return Ok(ScopedJson::Derived(Json::Object(new_object)));
            }
            _ => Err(RenderError::new("Object/Array not found")),
        }
    }
}

#[derive(Clone, Copy)]
pub struct StringifyHelper;

impl HelperDef for StringifyHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let param = h.param(0).map(|v| v.value()).expect("Expected parameter");
        out.write(Json::to_string(param).as_str())?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct DateFormaterHelper;

impl HelperDef for DateFormaterHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let date = h
            .param(0)
            .map(|v| v.value())
            .expect("Expected date parameter");
        let format = h
            .param(1)
            .map(|v| v.value())
            .expect("Expected format parameter");

        let date = match date {
            Json::Number(date) => date
                .as_i64()
                .map(|v| Utc.timestamp_millis_opt(v))
                .expect("Failed to parse date"),
            _ => return Err(RenderError::new("Date must be a string or number")),
        };

        let format = match format {
            Json::String(format) => format,
            _ => return Err(RenderError::new("Format must be a string")),
        };

        match date {
            LocalResult::Single(date) => {
                let date = date.format(format.as_str()).to_string();
                out.write(date.as_str())?;
            }
            _ => return Err(RenderError::new("Failed to parse date")),
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct SortByHelper;

impl HelperDef for SortByHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
        let object = h
            .param(0)
            .map(|v| v.value())
            .ok_or(RenderError::new("Array not found"))?;
        let object = match object {
            Json::Object(object) => object,
            _ => return Err(RenderError::new("Object not found")),
        };

        let sort_by = h
            .param(1)
            .map(|v| v.value())
            .ok_or(RenderError::new("Sort by not found"))?;
        let sort_by = match sort_by {
            Json::String(sort_by) => sort_by,
            _ => return Err(RenderError::new("Sort by must be a string")),
        };

        let order = h
            .param(2)
            .map(|v| v.value())
            .ok_or(RenderError::new("Order not found"))?;
        let order = match order {
            Json::String(order) => order,
            _ => return Err(RenderError::new("Order must be a string")),
        };

        if order != "asc" && order != "desc" {
            return Err(RenderError::new("Order must be either asc or desc"));
        }

        let mut sortable = object
            .iter()
            .map(|object| {
                let (_, value) = object;

                let value = match value {
                    Json::Object(object) => object,
                    _ => return Err(RenderError::new("Object not found")),
                };

                let key: String = value.get(sort_by).unwrap_or(&json!("")).to_string();

                Ok((key, object))
            })
            .collect::<Result<Vec<(String, (&String, &Json))>, RenderError>>()?;

        if order == "asc" {
            sortable.sort_by(|a, b| b.0.cmp(&a.0));
        } else {
            sortable.sort_by(|a, b| a.0.cmp(&b.0));
        }

        let mut sorted_object = Map::new();

        for (_, (key, value)) in sortable {
            sorted_object.insert(key.clone(), value.clone());
        }

        Ok(ScopedJson::Derived(Json::Object(sorted_object)))
    }
}
