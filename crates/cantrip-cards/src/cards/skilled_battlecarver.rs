use cantrip_api::*;
use cantrip_mechanics::{pump, statics};
use cantrip_oracle as oracle;

/// "During your turn, this creature has first strike." (CR 500.1, 613.1f, layer 6): its controller's turn, asked afresh
/// whenever characteristics are computed.
fn first_strike_during_your_turn(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = view.state();
    if state.turn.active == state.obj(source).controller {
        chars.keywords = chars.keywords.with(Keyword::FirstStrike);
    }
}

/// "{1}{R}: This creature gets +1/+0 until end of turn." (CR 611.2)
fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{R}"), pump_self, "{1}{R}: This creature gets +1/+0 until end of turn.")],
    statics: &[static_ability(Layer::Ability, statics::itself, first_strike_during_your_turn, "During your turn, this creature has first strike.")],
    ..oracle::SKILLED_BATTLECARVER
};
