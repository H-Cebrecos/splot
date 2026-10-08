# splot

An application to draw timing diagrams as text, mostly for HDL documentation comments

a made up example of how the diagrams are rendered

```
                                            ╭╴transfer
             ├──╴start╶──┼───╴handshake╶───┼┴────────┼───╴end╶───┤
        clk  ┌──┐  ┌──┐  ┌──┐  ┌──┐  ┌──┐  ┌──┐  ┌···┌──┐  ┌──┐  ┌···
                └──┘  └──┘  └──┘  └──┘  └──┘  └──┘   ┘  └──┘  └──┘
             ╎     ╭╴a   ╎  deassert╶╮     ╎         ╎     ╭╴ack ╎
        ctrl ╳▔▔▔▔▔╲▁▁▁▁▁╱▔▔▔▔▔▔▔▔▔▔▔╲▁▁▁▁▁╳░░░░░░···╳▔▔▔▔▔╲▁▁▁▁▁╳···
             ╎           ╰╴below           ╎     ╭╴b ╎           ╎
        data ╳░░░░░░░░░░░╳ ╶────╴h00╶────╴ ╳ hAF ╳···╳░░░░░░░░░░░╳···
             ╎           ╎                 ╎         ╎           ╎

        a: falling level indicates that the master is ready.
        b: as many transfers as needed at a rate of one trasfer per
           clock cycle.

```

# TODO:

- half cycle support for DDR style clocking

- long region names

- suport undefined start and ends for boundary regions ···─╴start╶─┼──*──┼──╴end╶───···
  with syntax start:+n | ... | end:+

- size the clock while hidden

# test deploy

```
wasm-pack build --target web
python -m http.server
```
