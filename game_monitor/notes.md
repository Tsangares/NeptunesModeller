## TODO

 - Make more datastructures.
 - Abstract the concepts of maintaining a single file using the ledger and many clients.
 - Figure out the entry point: Is there a way to listen/broadcast?
 - Maybe write an MQTT subscriber implement in IOTA_CLIENT
 - For this appliaction it does not bi-directional streams. The program runs periodically and updates the list if there does not include the game_id that we have in our cache.
 - Is the broadcast to recieve the last block? Yes => How do you go find a new block? You must have a MQTT stream listening and broadcasting it somewhere. Or I suppose all servers are listening to the MQTT stream. But then if one is off, how does it sync back up? 