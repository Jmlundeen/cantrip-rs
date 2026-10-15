//! Vote (CR 701.38a): "Starting with a specified player and proceeding in turn order, each player votes for one of the
//! given options." Only players still in the game vote (CR 800.4a). Council's Judgment prints it, voting for permanents
//! (will of the council).

use cantrip_api::*;

/// Each player still in the game, starting with `first` and proceeding in turn order, votes for one of `candidates`:
/// the votes each candidate got, in the order given. A single candidate takes every vote and nobody is asked.
pub fn for_permanents(g: &mut dyn Game, source: ObjectId, first: PlayerId, candidates: &[ObjectId]) -> Result<Vec<u32>, Suspend> {
    let voters: Vec<PlayerId> = g.state().turn_order_from(first).collect();
    let mut votes = vec![0u32; candidates.len()];
    if candidates.len() == 1 {
        votes[0] = voters.len() as u32;
        return Ok(votes);
    }
    for voter in voters {
        if let Some(card) = crate::choose::card(g, voter, source, Purpose::Vote, candidates.to_vec())?
            && let Some(i) = candidates.iter().position(|&c| c == card)
        {
            votes[i] += 1;
        }
    }
    Ok(votes)
}

/// The candidates tied for the most votes: none when nobody voted.
pub fn most_votes<T: Copy>(candidates: &[T], votes: &[u32]) -> Vec<T> {
    let most = votes.iter().copied().max().unwrap_or(0);
    if most == 0 {
        return Vec::new();
    }
    candidates.iter().zip(votes).filter(|&(_, &v)| v == most).map(|(&c, _)| c).collect()
}
