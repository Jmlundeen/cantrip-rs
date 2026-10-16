use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

fn gains_trample(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = Keywords(chars.keywords.0 | (1 << Keyword::Trample as u8));
}

/// "Whenever equipped creature deals combat damage to a player": the creature this is attached to dealing combat
/// damage to a player (CR 510.2); each such event is one player and one trigger (CR 603.2c).
fn equipped_deals_combat_damage_to_a_player(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: dealer, target: Target::Player(_), amount, combat: true }
            if *amount > 0 && q.attached_to(source) == Some(*dealer) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Choose a creature type. Create a Treasure token for each creature you control of that type": the type is chosen
/// as the ability resolves (CR 608.2d), the creatures counted after, and the Treasures created together (CR 101.4).
fn treasures_for_type(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let ask = RequestKind::ChooseSubtype { source: item.object, choices: Subtype::CREATURE_TYPES.to_vec() };
    let Answer::Subtype(chosen) = g.ask(you, ask)? else { return Ok(()) };
    let count = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Creature) && c.has_subtype(chosen)
        })
        .count();
    if count == 0 {
        return Ok(());
    }
    let treasure = g.card_id(&tokens::TREASURE);
    let all: Vec<(CardId, PlayerId)> = (0..count).map(|_| (treasure, you)).collect();
    g.create_tokens_together(&all)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::DAMAGE, equipped_deals_combat_damage_to_a_player), treasures_for_type, "Whenever equipped creature deals combat damage to a player, choose a creature type. Create a Treasure token for each creature you control of that type."),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_two_plus_two, "Equipped creature gets +2/+2."),
        static_ability(Layer::Ability, attached_object, gains_trample, "Equipped creature has trample."),
    ],
    ..oracle::ORCRIST_GOBLIN_CLEAVER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
