# Preview profile test art

Dev-only. These files back the fake members in the `#profile-tests` channel of
`yapfast --preview` (`crates/yapfast-core/src/application/preview/test_members.rs`),
so profile features can be tested without opening real people's profiles or using
an account. Don't put this directory or that module in an upstream PR without
asking first.

- `make_art.py` draws everything here with PIL: original coloured shapes, nothing
  from Discord or its cache. Run `python3 make_art.py` to write the files again;
  it checks each animated PNG's `acTL` frame and play counts as it goes.
- The moving nameplate is the shared `../gif-vp9-alpha.webm`.
- The preview reads these at runtime from the checkout it was built in, and
  decodes them with `decode_picture` as downloaded pictures are, handing them on as
  `Event::Picture` does. Nothing is fetched, and the fake addresses use the reserved
  `preview-profiles.invalid` domain. A build run without this directory shows
  initials and no effects.

## The effects, and what's right

Effects fade while the card is pointed at, and start half a second after it opens.

| Member | Layers | Should look like |
| --- | --- | --- |
| Effect: burst then loop | `effect-burst.png` (loop, no delay, `plays=1`, 800 ms, ends clear); `effect-stars.png` (loop, starts at 5000 ms, 2000 ms, delay 3000 ms, `plays=0`) | The burst goes off once and never returns. Stars first come about 5 s in, then every 5 s. |
| Effect: looping | `effect-loop.png` (loop, no delay, `plays=0`, 1000 ms) | Bubbles rise for ever. |
| Effect: plays-once layer | `effect-once.png` (no loop, `plays=1`, 1200 ms) | A bar goes down once, then nothing. |
