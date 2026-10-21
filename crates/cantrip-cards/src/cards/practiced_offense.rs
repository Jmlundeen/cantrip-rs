use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::flashback;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature target player controls (nothing if that target is no longer legal), then the
/// caster chooses double strike or lifelink for the target creature (CR 608.2d), which gains it until end of turn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(player) = item.target(0)
    {
        let creatures: Vec<ObjectId> =
            g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
        for creature in creatures {
            g.add_counters(creature, CounterKind::PlusOne, 1);
        }
    }
    if let Some(creature) = targets::object(g, item, 1) {
        let card = g.obj(item.object).card;
        let chosen = choose::one_of(g, item.controller, item.object, vec![Label::Card(card, 0), Label::Card(card, 1)])?;
        let keyword = if chosen == 1 { Keyword::Lifelink } else { Keyword::DoubleStrike };
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[keyword])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER, targets::CREATURE], effect),
    labels: &["Double strike", "Lifelink"],
    ways: &[flashback::way(cost("{1}{W}"), "Flashback {1}{W}")],
    ..oracle::PRACTICED_OFFENSE
};
