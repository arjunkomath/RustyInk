use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    ScopedJson,
};
use serde_json::value::Value as Json;

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
