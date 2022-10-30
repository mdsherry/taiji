# Taiji Tool

I really enjoyed playing [Taiji](https://taiji-game.com/), but drawing puzzles out on graph paper got tiring so I built a small tool to help solve some of the puzzles.

If you have not played Taiji without the tool, do that before using it. A significant portion of the challenge comes from figuring out puzzle mechanics from context, and the tool will explicitly tell you what constraints are being violated in each puzzle. The tool is far more useful in later portions of the game.

Although the main goal is to provide a more sophisticated alternative to the in-game puzzle board, or pencil and paper, I have also added an automatic solver. It will try to brute-force the puzzle, although there are a few smarts to prune paths that are obviously invalid. It can handle puzzles up to 5×5 fine, but will probably take too long for anything larger.

