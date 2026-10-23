use cantrip_api::*;
use cantrip_mechanics::{library, mana, select};
use cantrip_oracle as oracle;

/// "Up to two basic land cards that share a land type": each chosen pair must share at least one land type (CR 205.3i,
/// 701.23b).
fn shares_land_type(a: SubtypeSet, b: SubtypeSet) -> bool {
    SubtypeSet::LAND_TYPES.iter().any(|t| a.has(t) && b.has(t))
}

fn share_a_land_type(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let shares =
        |a: ObjectId, b: ObjectId| shares_land_type(q.chars(a).subtypes, q.chars(b).subtypes);
    let complete = choice.selected.len() < 2 || shares(choice.selected[0], choice.selected[1]);
    let addable = Bits::of(choice.candidates.len(), |i| {
        choice
            .selected
            .iter()
            .all(|&selected| shares(selected, choice.candidates[i]))
    });
    Verdict { complete, addable }
}

const SHARES_LAND_TYPE: Part = select::part(
    &(share_a_land_type as SelectFn),
    0,
    "that share a land type",
);
const SEARCH: Selection =
    Selection::of(select::up_to(2), &[select::BASIC_LAND, SHARES_LAND_TYPE]).called("basic land");

/// The search is one or zero cards when no pair shares a land type (CR 701.23b); the selection itself enforces the
/// relationship as each card is chosen, so two incompatible basic lands cannot both be found.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, SEARCH, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{2}, {T}").and(SACRIFICE_THIS),
            search,
            "{2}, {T}, Sacrifice this land: Search your library for up to two basic land cards that share a land type, put them onto the battlefield tapped, then shuffle.",
        ),
    ],
    ..oracle::MYRIAD_LANDSCAPE
};

#[cfg(test)]
mod tests {
    use super::shares_land_type;
    use cantrip_api::{Subtype, SubtypeSet};

    #[test]
    fn shared_land_type_includes_nonbasic_land_types() {
        let desert = SubtypeSet::of(&[Subtype::Desert]);
        let gate = SubtypeSet::of(&[Subtype::Gate]);
        assert!(shares_land_type(desert, desert));
        assert!(!shares_land_type(desert, gate));
    }
}
