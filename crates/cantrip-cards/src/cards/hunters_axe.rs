use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{choose, statics};
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, it gains your choice of trample or deathtouch until end of turn."
fn gain_choice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = item.object;
    let axe = g.card_id(&CARD);
    let keyword = match choose::one_of(g, item.controller, creature, vec![Label::Card(axe, 0), Label::Card(axe, 1)])? {
        0 => Keyword::Trample,
        _ => Keyword::Deathtouch,
    };
    g.add_effect(creature, creature, EffectKind::AddKeywords(Keywords::of(&[keyword])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    delayed: &[triggered(
        cantrip_mechanics::triggers::THIS_ATTACKS,
        gain_choice,
        "Whenever this creature attacks, it gains your choice of trample or deathtouch until end of turn.",
    )],
    statics: &[
        static_ability(Layer::PtModify, attached_object, statics::plus_pt::<2, 0>, "Equipped creature gets +2/+0."),
        static_ability(
            Layer::Ability,
            attached_object,
            statics::grant_delayed_ability,
            "Equipped creature has \"Whenever this creature attacks, it gains your choice of trample or deathtouch until end of turn.\"",
        ),
    ],
    labels: &["Trample", "Deathtouch"],
    ..oracle::HUNTERS_AXE
};
