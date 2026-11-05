
use crate::time::*;

struct Task {
    // the essential attributes
    id:             u64,
    title:          String,
    description:    String,
    date:           Date,
    // optional attributes
    start_time:     Option<Time>,
    duration:       Option<Duration>,
    recurrence:     Option<Recurrence>,
    // completion status
    completed:      bool,
}

impl Task {
    // ---- all getters ----
    pub fn id(&self) -> u64 { self.id }
    pub fn title(&self) -> &str { &self.title }
    pub fn description(&self) -> &str { &self.description }
    pub fn date(&self) -> &Date { &self.date }
    
    // as_ref() converts from &Option<Type> to Option<&Type>, meaning it's perfect for this
    pub fn start_time(&self) -> Option<&Time> {
        self.start_time.as_ref()
    }
    pub fn duration(&self) -> Option<&Duration> {
        self.duration.as_ref()
    }
    pub fn recurrence(&self) -> Option<&Recurrence> {
        self.recurrence.as_ref()
    }

    pub fn completed(&self) -> bool { self.completed }

}

