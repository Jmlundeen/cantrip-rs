use cantrip_api::*;
use cantrip_mechanics::{choose, choose_color_for, effects, statics};
use cantrip_oracle as oracle;

/// As this enchantment enters, its entering controller chooses a color and an opponent (CR 614.12a).
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.chosen_color = Some(choose_color_for(g, entering.controller, entering.object, ColorUse::Any)?);
    let opponents = g.state().opponents(entering.controller).collect();
    entering.chosen_player = choose::player(g, entering.controller, entering.object, opponents, "is the chosen player", 0)?;
    Ok(())
}

/// Whether the chosen player controls a nontoken permanent of the chosen color (CR 109.5), as `source` chose them as it
/// entered (CR 614.12a, 607.2d). `owner_and_colors` says who controls a permanent and what colors it is, as the caller reads
/// them (through the layers, or as the game stands).
fn chosen_player_controls_chosen_color(
    s: &GameState,
    link: Linked,
    owner_and_colors: &dyn Fn(ObjectId) -> (PlayerId, ColorSet),
) -> bool {
    let (Some(player), Some(color)) = (s.chosen_player(link), s.chosen_color(link)) else { return false };
    s.battlefield().iter().any(|&id| {
        let (controller, colors) = owner_and_colors(id);
        controller == player && !s.obj(id).token && colors.has(color)
    })
}

/// "White creatures get +2/+1 as long as the chosen player controls a nontoken permanent of the chosen color." (CR 604.1).
fn white_creatures_get_2_1_affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    chosen_player_controls_chosen_color(q.state(), q.linked_to(source), &|id| (q.controller(id), q.chars(id).colors))
        && statics::creatures_of_color::<{ Color::White as u8 }>(q, source, affected, so_far)
}

/// When the chosen player controls no nontoken permanents of the chosen color (CR 603.8).
fn as_enchantment_enters_choose_color_state(q: &dyn Query, asker: &Asker) -> bool {
    !chosen_player_controls_chosen_color(q.state(), q.linked_to(asker), &|id| (q.controller(id), q.chars(id).colors))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        state_triggered(
            as_enchantment_enters_choose_color_state,
            effects::sacrifice_self,
            "When the chosen player controls no nontoken permanents of the chosen color, sacrifice this enchantment.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            white_creatures_get_2_1_affects,
            statics::plus_pt::<2, 1>,
            "White creatures get +2/+1 as long as the chosen player controls a nontoken permanent of the chosen color.",
        ),
    ],
    enters: Some(enters),
    ..oracle::JIHAD
};
