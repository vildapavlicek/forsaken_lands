use {
    crate::{ResearchCompleted, ResearchLibrary, ResearchState, StartResearchRequest},
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

        // 1. Retrieval
        let Some(def) = library.available.get(research_id) else {
            warn!("Research ID {} not found.", research_id);
            continue;
        };

        // 2. Execution (UI already validated costs, etc.)
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
    }
}

pub fn update_research_progress(
    time: Res<Time>,
    mut state: ResMut<ResearchState>,
    mut commands: Commands,
) {
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
        state.completed.push(id.clone());
        // Optional: Send event `ResearchCompleted(id)` for UI popups
        commands.trigger(ResearchCompleted {
            research_id: id,
        });
    }
}
