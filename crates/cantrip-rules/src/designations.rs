//! Player designations and the dungeon: the monarch (CR 725), the initiative (CR 726) and venturing into the Undercity
//! (CR 701.49, 309). Their abilities have no source object: the rules put them on the stack themselves, from an
//! object of the rules' own that is in no zone.

use cantrip_api::intrinsic::{INITIATIVE_TAKEN, INITIATIVE_VENTURE, MONARCH_DRAW, MONARCH_TAKEN};
use cantrip_api::*;
use cantrip_state::{PendingTrigger, Span};

use crate::{Cx, R};

/// Puts an ability of the rules' own on its way to the stack (CR 603.3), controlled by `controller` and about
/// `subject`. Its source is a new object in no zone: it ceases to matter once the ability resolves.
fn queue(cx: &mut Cx, card: CardId, index: u8, controller: PlayerId, subject: Target) {
    let source = cx.st().push_obj(Obj::new(card, controller, Zone::GONE));
    cx.st().trigger_push(PendingTrigger {
        source,
        ability: AbilityRef { card, index },
        controller,
        subject,
        data: [0; 2],
        members: Span::default(),
    });
}

fn in_game(cx: &Cx, player: PlayerId) -> bool {
    !cx.s().player(player).lost
}

/// A player becomes the monarch (CR 725.1); whoever was stops being it (CR 725.3). Nothing happens to one who is it
/// already or who has left the game.
pub fn become_monarch(cx: &mut Cx, player: PlayerId) {
    if !in_game(cx, player) || cx.s().player(player).monarch {
        return;
    }
    set_monarch(cx, Some(player));
}

fn set_monarch(cx: &mut Cx, monarch: Option<PlayerId>) {
    for p in cx.s().seats().collect::<Vec<_>>() {
        let is = Some(p) == monarch;
        cx.st().update_player(p, |pl| pl.monarch = is);
    }
    // What watches a player becoming the monarch (Palace Jailer's wait) sees it (CR 725.1).
    if let Some(player) = monarch {
        cx.emit(Event::BecameMonarch { player });
    }
}

/// A monarch who has left the game hands the designation to the active player, or if that is who left, the next
/// player in turn order still in the game (CR 725.4).
pub fn monarch_left(cx: &mut Cx) {
    let Some(leaver) = cx.s().seats().find(|&p| cx.s().player(p).monarch && cx.s().player(p).lost) else { return };
    let order: Vec<PlayerId> = cx.s().apnap().collect();
    let heir = order.into_iter().find(|&p| p != leaver && in_game(cx, p));
    set_monarch(cx, heir);
}

/// The monarch's two inherent triggers (CR 725.2), checked as a step begins: "at the beginning of the monarch's end
/// step, that player draws a card".
pub fn step_began(cx: &mut Cx, step: Step) {
    let active = cx.s().turn.active;
    let intrinsic = cx.cards().intrinsic;
    if step == Step::End && cx.s().player(active).monarch {
        queue(cx, intrinsic, MONARCH_DRAW, active, Target::None);
    }
    // "At the beginning of the upkeep of the player who has the initiative, that player ventures into Undercity."
    if step == Step::Upkeep && cx.s().player(active).initiative {
        queue(cx, intrinsic, INITIATIVE_VENTURE, active, Target::None);
    }
}

/// Combat damage dealt to the monarch or to the player with the initiative (CR 725.2, 726.2): "whenever a creature
/// deals combat damage to the monarch, its controller becomes the monarch"; "whenever one or more creatures a player
/// controls deal combat damage to the player who has the initiative, the controller of those creatures takes the
/// initiative", once for each such player however many creatures that was.
pub fn combat_damage(cx: &mut Cx, source: ObjectId, victim: PlayerId, amount: u16) {
    let player = *cx.s().player(victim);
    if amount == 0 || !(player.monarch || player.initiative) {
        return;
    }
    // A creature, as it last existed (CR 603.10): it may have died dealing the damage.
    let creature = if cx.s().obj(source).zone.is(ZoneKind::Battlefield) {
        crate::layers::chars(cx.cards(), cx.s(), source).is(CardType::Creature)
    } else {
        cx.s().last_known(source).is_some_and(|l| l.types.has(CardType::Creature))
    };
    if !creature {
        return;
    }
    let controller = cx.s().obj(source).controller;
    let intrinsic = cx.cards().intrinsic;
    let subject = Target::Player(controller);
    if player.monarch {
        queue(cx, intrinsic, MONARCH_TAKEN, victim, subject);
    }
    if player.initiative {
        let waiting = cx.s().triggers.iter().any(|t| {
            t.ability == AbilityRef { card: intrinsic, index: INITIATIVE_TAKEN } && t.subject == subject
        });
        if !waiting {
            queue(cx, intrinsic, INITIATIVE_TAKEN, victim, subject);
        }
    }
}

/// A player takes the initiative (CR 726.1): whoever had it stops (CR 726.3), and the taker ventures into Undercity
/// (CR 726.2), even one who already had it (CR 726.5).
pub fn take_initiative(cx: &mut Cx, player: PlayerId) {
    if !in_game(cx, player) {
        return;
    }
    for p in cx.s().seats().collect::<Vec<_>>() {
        let is = p == player;
        cx.st().update_player(p, |pl| pl.initiative = is);
    }
    let intrinsic = cx.cards().intrinsic;
    queue(cx, intrinsic, INITIATIVE_VENTURE, player, Target::None);
}

/// Venturing into the Undercity (CR 701.49): enter it at its first room; or from a room move on along an arrow, the
/// player's choice where there are two (CR 309.5a); or from the bottommost room complete it (CR 309.7) and enter it
/// again (CR 701.49c). The room moved into triggers its ability (CR 309.4c).
pub fn venture(cx: &mut Cx, player: PlayerId) -> R<()> {
    let Some(dungeon) = cx.cards().undercity else { return Ok(()) };
    if !in_game(cx, player) {
        return Ok(());
    }
    let def = cx.card(dungeon);
    let here = cx.s().player(player).room;
    let mut room = 0u8;
    if here != 0 {
        let arrows = def.dungeon[here as usize - 1];
        match arrows.len() {
            0 => {
                // Out of the bottommost room: completed, and entered again from the top.
                cx.st().update_player(player, |p| p.dungeons_completed = p.dungeons_completed.saturating_add(1));
            }
            1 => room = arrows[0],
            _ => {
                let source = cx.s().stack.last().map_or(ObjectId(0), |i| i.object);
                let modes: Vec<Label> = arrows.iter().map(|&r| Label::Card(dungeon, r)).collect();
                let eligible: Vec<u8> = (0..arrows.len() as u8).collect();
                let ask = RequestKind::ChooseModes { source, modes, eligible, count: 1, at_most: 0, affordable: Vec::new(), additional_cost: false, repeat: false };
                let chosen = match cx.ask_core(player, ask, false)? {
                    Answer::Modes(m) => m.first().copied().unwrap_or(0) as usize,
                    _ => 0,
                };
                room = arrows[chosen.min(arrows.len() - 1)];
            }
        }
    }
    cx.st().update_player(player, |p| p.room = room + 1);
    queue(cx, dungeon, room, player, Target::None);
    Ok(())
}

/// State-based action (CR 704.5t): a player whose venture marker is on the bottommost room of a dungeon, with none of
/// that room's abilities on the stack, completes it (CR 309.7). Whether anyone did.
pub fn complete_dungeons(cx: &mut Cx) -> bool {
    let Some(dungeon) = cx.cards().undercity else { return false };
    let def = cx.card(dungeon);
    let mut any = false;
    for p in cx.s().seats().collect::<Vec<_>>() {
        let room = cx.s().player(p).room;
        if room == 0 || !def.dungeon[room as usize - 1].is_empty() {
            continue;
        }
        if cx.s().stack.iter().any(|i| i.kind == StackKind::Triggered && i.ability.card == dungeon && i.controller == p) {
            continue;
        }
        cx.st().update_player(p, |pl| {
            pl.room = 0;
            pl.dungeons_completed = pl.dungeons_completed.saturating_add(1);
        });
        any = true;
    }
    any
}
