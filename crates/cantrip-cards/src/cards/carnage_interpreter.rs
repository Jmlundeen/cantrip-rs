use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Discard your hand (together, nothing chosen), then investigate four times.
fn discard_and_investigate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.discard_hand(item.controller);
    for _ in 0..4 {
        tokens::investigate(g, item.controller)?;
    }
    Ok(())
}

/// "As long as you have one or fewer cards in hand": only this creature, while its controller's hand allows.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    source == affected && state.zone(Zone::hand(state.obj(source).controller)).len() <= 1
}

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

fn menace(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Menace);
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, discard_and_investigate, "When this creature enters, discard your hand, then investigate four times.")],
    statics: &[
        StaticDef {
            text: "As long as you have one or fewer cards in hand, this creature gets +2/+2.",
            layer: Layer::PtModify,
            affects,
            apply: plus_two_plus_two,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "As long as you have one or fewer cards in hand, this creature has menace.",
            layer: Layer::Ability,
            affects,
            apply: menace,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::CARNAGE_INTERPRETER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
