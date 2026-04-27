use std::hash::Hash;

use ae::pf::{InData, OutData, OutFlags, ParamUIFlags, Parameters};
use after_effects as ae;

pub type UiRule<Params> = (Params, bool);

pub fn enable_update_params_ui(out_data: &mut OutData) {
    out_data.set_out_flag(OutFlags::SendUpdateParamsUi, true);
}

pub fn apply_premiere_invisible<ParamsT>(
    params: &mut Parameters<ParamsT>,
    rules: &[UiRule<ParamsT>],
) -> Result<(), ae::Error>
where
    ParamsT: Copy + Eq + PartialEq + Hash + std::fmt::Debug,
{
    let mut cloned = params.cloned();
    for (param, visible) in rules {
        let mut pd = cloned.get_mut(*param)?;
        pd.set_ui_flag(ParamUIFlags::INVISIBLE, !visible);
        pd.update_param_ui()?;
    }
    Ok(())
}

pub fn apply_disabled<ParamsT>(
    params: &mut Parameters<ParamsT>,
    rules: &[UiRule<ParamsT>],
) -> Result<(), ae::Error>
where
    ParamsT: Copy + Eq + PartialEq + Hash + std::fmt::Debug,
{
    let mut cloned = params.cloned();
    for (param, enabled) in rules {
        let mut pd = cloned.get_mut(*param)?;
        pd.set_ui_flag(ParamUIFlags::DISABLED, !enabled);
        pd.update_param_ui()?;
    }
    Ok(())
}

pub fn apply_after_effects_hidden<ParamsT>(
    in_data: InData,
    plugin_id: ae::aegp::PluginId,
    params: &Parameters<ParamsT>,
    rules: &[UiRule<ParamsT>],
) -> Result<(), ae::Error>
where
    ParamsT: Copy + Eq + PartialEq + Hash + std::fmt::Debug,
{
    if plugin_id == 0 {
        return Ok(());
    }

    let effect = in_data.effect();
    let aegp_effect = effect.aegp_effect(plugin_id)?;

    for (param, visible) in rules {
        let index = params.index(*param).ok_or(ae::Error::InvalidIndex)? as i32;
        aegp_effect
            .new_stream_by_index(plugin_id, index)?
            .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, !visible)?;
    }

    Ok(())
}
