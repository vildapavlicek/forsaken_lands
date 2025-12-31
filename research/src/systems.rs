use {
    crate::{ResearchLibrary, ResearchState, StartResearchRequest},
    bevy::prelude::*,
    wallet::Wallet,
};

pub fn start_research(
    mut events: MessageReader<StartResearchRequest>,
    library: Res<ResearchLibrary>,
    mut state: ResMut<ResearchState>,
    mut wallet: ResMut<Wallet>,
) {
    for event in events.read() {
        let research_id = &event.0;

        // 1. Validation
        let Some(def) = library.available.get(research_id) else {
            warn!("Research ID {} not found.", research_id);
            continue;
        };

        if state.is_researched(research_id) || state.is_researching(research_id) {
            continue;
        }

        // Check prerequisites
        if !def.prerequisites.iter().all(|req| state.is_researched(req)) {
            info!("Prerequisites not met for {}", def.name);
            continue;
        }

        // Check costs
        let can_afford = def
            .cost
            .iter()
            .all(|(res, amt)| wallet.resources.get(res).copied().unwrap_or(0) >= *amt);

        // 2. Execution
        if can_afford {
            // Deduct Resources
            for (res, amt) in &def.cost {
                if let Some(resource_amt) = wallet.resources.get_mut(res) {
                    *resource_amt -= amt;
                }
            }

            // Init Timer (Mode::Once)
            state.in_progress.insert(
                research_id.clone(),
                Timer::from_seconds(def.time_required, TimerMode::Once),
            );
            info!("Started researching: {}", def.name);
        } else {
            info!("Cannot afford research: {}", def.name);
        }
    }
}

pub fn update_research_progress(time: Res<Time>, mut state: ResMut<ResearchState>) {
    let mut finished = Vec::new();

    for (id, timer) in state.in_progress.iter_mut() {
        timer.tick(time.delta());
        if timer.is_finished() {
            finished.push(id.clone());
        }
    }

    for id in finished {
        info!("Research complete: {}", id);
        state.in_progress.remove(&id);
        state.completed.push(id);
        // Optional: Send event `ResearchCompleted(id)` for UI popups
    }
}
