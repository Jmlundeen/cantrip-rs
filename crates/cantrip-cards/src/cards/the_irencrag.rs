use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// The name it becomes (CR 612.8).
const EVERFLAME: &str = "Everflame, Heroes' Legacy";

const ASK: Question = Question::new(
    "BecomeEquipment",
    "become a legendary Equipment artifact named Everflame, Heroes' Legacy?",
    "It gains equip {3} and \"Equipped creature gets +3/+3\" and loses all other abilities",
    "Become Everflame",
    "Don't",
);

/// "You may have The Irencrag become a legendary Equipment artifact named Everflame, Heroes' Legacy. If you do, it gains
/// equip {3} and 'Equipped creature gets +3/+3' and loses all other abilities." With no duration, for as long as it stays
/// on the battlefield (the ruling, CR 611.2a). The name is set by a text-changing effect (CR 612.8, layer 3), so it has no
/// other name; "becomes a legendary Equipment artifact" sets its card types to artifact alone and its artifact types to
/// Equipment alone (CR 205.1a, layer 4; it is legendary already); and in layer 6 it loses its abilities and then gains the two, which come later
/// (CR 613.7). "The Irencrag" is this object (CR 201.5): one that has left the battlefield is a new object and is not
/// changed (CR 400.7).
fn become_everflame(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let me = item.object;
    let card = g.card_id(&CARD);
    let forever = Duration::Indefinite;
    instruction(g, |g| {
        g.add_effect(me, me, EffectKind::SetName(EVERFLAME), forever);
        g.add_effect(me, me, EffectKind::SetTypes(TypeSet::of(&[CardType::Artifact])), forever);
        g.add_effect(me, me, EffectKind::LoseSubtypesOf(CardType::Artifact), forever);
        g.add_effect(me, me, EffectKind::AddSubtype(Subtype::Equipment), forever);
    });
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    instruction(g, |g| {
        g.add_effect(me, me, EffectKind::LoseAllAbilities, forever);
        g.add_effect(me, me, EffectKind::AddAbility(AbilityRef::delayed(card, 0)), forever);
        g.add_effect(me, me, EffectKind::AddAbility(AbilityRef::delayed(card, 1)), forever);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        triggered(
            cantrip_mechanics::triggers::LEGENDARY_CREATURE_YOU_CONTROL_ENTERS,
            become_everflame,
            "Whenever a legendary creature you control enters, you may have The Irencrag become a legendary Equipment artifact named Everflame, Heroes' Legacy. If you do, it gains equip {3} and \"Equipped creature gets +3/+3\" and loses all other abilities.",
        ),
    ],
    delayed: &[
        equip(mana("{3}"), "Equip {3}"),
        granted_static_ability(Layer::PtModify, attached_object, plus_pt::<3, 3>, "Equipped creature gets +3/+3"),
    ],
    ..oracle::THE_IRENCRAG
};
