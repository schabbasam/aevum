
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


