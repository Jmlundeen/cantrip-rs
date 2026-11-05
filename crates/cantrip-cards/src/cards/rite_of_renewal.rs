use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Up to four target cards from their graveyard": in the graveyard of the player an earlier slot chose (CR 115.1, 601.2c),
/// not merely in some graveyard.
fn card_in_their_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    let Some(Target::Player(player)) = q.chosen_targets().iter().copied().find(|chosen| matches!(chosen, Target::Player(_))) else { return false };
    q.obj(id).zone == Zone::graveyard(player) && q.obj(id).became.is_none()
}

/// "Return up to two target permanent cards from your graveyard to your hand. Target player shuffles up to four target cards
/// from their graveyard into their library. Exile Rite of Renewal." Each sentence is its own instruction (CR 608.2c): the cards
/// of the first slot that are still legal come back together (CR 608.2b), and one that is also targeted by the third slot is
/// in a hand by the time that is read, so it is not shuffled. The player shuffles even if none of the cards remains
/// (CR 701.24a), as long as they are still a legal target; the spell exiles itself last (CR 608.2m).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let returning: Vec<ObjectId> = g.state().spec_targets(item, 0).filter_map(|slot| targets::object(g, item, slot)).collect();
    g.return_to_hand(&returning);
    g.end_sentence();
    if let Some(player) = g.state().spec_targets(item, 1).find_map(|slot| targets::player(g, item, slot)) {
        let shuffling: Vec<ObjectId> = g.state().spec_targets(item, 2).filter_map(|slot| targets::object(g, item, slot)).collect();
        g.put_into_library(&shuffling, LibraryPlace::Top);
        g.shuffle(player);
    }
    g.end_sentence();
    g.exile(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD.between(0, 2), targets::PLAYER, target(card_in_their_graveyard).between(0, 4)], effect),
    ..oracle::RITE_OF_RENEWAL
};
