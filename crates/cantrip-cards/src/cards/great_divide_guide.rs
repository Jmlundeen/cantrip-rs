use cantrip_api::*;
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_oracle as oracle;

/// "Each land and Ally you control" (CR 613.1f, layer 6): a land or an Ally under the source's controller's control, judged on
/// what the earlier layers made it (a land may be an Ally by an effect, and an Ally need not be a creature), the Ally word read
/// through the source's text (CR 612.2). The Guide is itself an Ally and has the ability it gives.
fn land_or_ally_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = view.state();
    s.obj(affected).controller == s.obj(source).controller
        && (so_far.is(CardType::Land) || so_far.has_subtype(view.words(source).subtype(Subtype::Ally)))
}

pub static CARD: CardDef = CardDef {
    delayed: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    statics: &[static_ability(
        Layer::Ability,
        land_or_ally_you_control,
        grant_delayed_ability,
        "Each land and Ally you control has \"{T}: Add one mana of any color.\"",
    )],
    ..oracle::GREAT_DIVIDE_GUIDE
};
