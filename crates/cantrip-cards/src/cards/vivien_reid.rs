use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "+1: Look at the top four cards of your library. You may reveal a creature or land card from among them and put it into
/// your hand. Put the rest on the bottom of your library in a random order." (CR 401.2, 701.20.)
fn look_and_take(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 4, (0, 1), true, |c| c.is(CardType::Creature) || c.is(CardType::Land), Rest::BottomRandom)?;
    Ok(())
}

/// "Target artifact, enchantment, or creature with flying" (CR 115.1), read from what the layers give it now.
fn destroyable(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT_OR_ENCHANTMENT.filter)(q, asker, t) || (targets::CREATURE_WITH_FLYING.filter)(q, asker, t)
}

// --- The emblem ---------------------------------------------------------------------------------------------------

/// "Creatures you control" (CR 114.4: the emblem's controller), from the command zone.
fn creatures_you_control(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    view.state().obj(affected).controller == view.state().obj(source).controller && chars.is(CardType::Creature)
}

/// "+2/+2" (layer 7c). CR 613.4b: a modifier only reaches something that has power and toughness to modify.
fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

/// "Vigilance, trample, and indestructible" (layer 6).
fn vigilance_trample_indestructible(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Vigilance).with(Keyword::Trample).with(Keyword::Indestructible);
}

pub static EMBLEM: CardDef = CardDef {
    name: "Vivien Reid emblem",
    has_cost: false,
    statics: &[
        static_ability(Layer::PtModify, creatures_you_control, plus_two_plus_two, "Creatures you control get +2/+2"),
        static_ability(Layer::Ability, creatures_you_control, vigilance_trample_indestructible, "and have vigilance, trample, and indestructible."),
    ],
    text: "Creatures you control get +2/+2 and have vigilance, trample, and indestructible.",
    ..CardDef::BLANK
};

/// \u{2212}8: "You get an emblem with \"Creatures you control get +2/+2 and have vigilance, trample, and indestructible.\""
fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), look_and_take, "+1: Look at the top four cards of your library. You may reveal a creature or land card from among them and put it into your hand. Put the rest on the bottom of your library in a random order."),
        ability(loyalty(-3), effects::destroy_first_target, "\u{2212}3: Destroy target artifact, enchantment, or creature with flying.")
            .targets(&[target(destroyable)]),
        ability(loyalty(-8), emblem, "\u{2212}8: You get an emblem with \"Creatures you control get +2/+2 and have vigilance, trample, and indestructible.\""),
    ],
    ..oracle::VIVIEN_REID
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
