What is this:
This is a graph viewer written in Rust with Iced.


TODO:
1. Failed algo paint red
3. Add current node highlighting
5. Fix bridge highlighting
6. Fix modal window reseting camera position
8. Fix rendering order (text is above arrows)
9. Finally zoom to center instead of top left corner

It can:
1. Load graphs
2. Display them
3. Save graphs
4. Create graphs
6. Display algorithms step-by-step 

roadmap:
7. Refactor
    - reconsider "drawable" interface
      - substitute component naming
      - possibly optimize drawing with cache
    - reconsider graph & camera structs + impl
      - possibly use default rust treats (into, ..)
8. Bench & optimizations
    - test windows compilation
    - possibly rendering optimizations
9. Release (create proper readme, create examples) - after report
    - compile release binaries
    - create useful readme
      - add iced badge =)
      - add screenshots
    - compose iced review (note problems)