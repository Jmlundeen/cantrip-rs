use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::replacements::{enter_tapped, permanents_enter_tapped};
use cantrip_mechanics::statics::{self, set_land_type};
use cantrip_oracle as oracle;

/// "{7}: Put a conqueror counter on Zhao." The counter goes only on Zhao while he is still on the battlefield (CR 400.7).
fn conqueror_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Conqueror, 1);
    Ok(())
}

/// "Nonbasic lands" (CR 205.4a, 613.1d; `statics::nonbasic_land`), of every player, "as long as Zhao has a conqueror counter on him" (CR 122.1): judged as
/// layers 1 to 3 left each land, and only while Zhao is on the battlefield with one.
fn nonbasic_lands_while_conquering(view: &dyn LayerQuery, source: ObjectId, object: ObjectId, so_far: &Chars) -> bool {
    view.state().obj(source).counters(CounterKind::Conqueror) >= 1 && statics::nonbasic_land(view, source, object, so_far)
}

/// "Nonbasic lands enter tapped" (CR 614.1d), of every player's, is `permanents_enter_tapped`; another effect that lets one enter
/// untapped "unless ..." changes nothing: it is tapped either way (the card's ruling). "... are Mountains" (CR 305.7, layer 4) is
/// `set_land_type`, as Blood Moon has it: the land loses its other land types and its rules text's abilities, and has "{T}: Add {R}"
/// (CR 305.6); its name and supertypes stay (the card's ruling).
/// Menace is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{7}"), conqueror_counter, "{7}: Put a conqueror counter on Zhao.")],
    replacements: &[permanents_enter_tapped::<{ enter_tapped::NONBASIC_LANDS }>("enters tapped")],
    statics: &[sets_land_subtypes(
        nonbasic_lands_while_conquering,
        set_land_type::<{ lands::sub(Subtype::Mountain) }>,
        "As long as Zhao has a conqueror counter on him, nonbasic lands are Mountains.",
    )],
    ..oracle::ZHAO_THE_MOON_SLAYER
};
