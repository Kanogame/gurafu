What is this:
This is a graph viewer written in Rust with Iced.

It can:
1. Load graphs
2. Display them
3. Save graphs
4. Create graphs
6. Display algorithms step-by-step 

roadmap:
4. Add timeline+highliting
    +/- create step-based algo - needs refactoring
    + add auto-stepper
    - add stepper
    - time chooser for stepper
    - add "to end" functionality
    - reset algo
    - better highlighting for algo
    - add messageboxes (or and notifs) about algo status - "euler path found"

5. add import/export via file
    - add json serialization/deserialization
    - save/open json files

6. UI rework
    - add Ids to nodes
    - add text helpers
7. Refactor
8. Bench & optimizations
9. Release