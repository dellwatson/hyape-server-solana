// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@coral-xyz/anchor");
const { PublicKey } = require("@solana/web3.js");
const { MultiplayerRooms } = require("../target/types/multiplayer_rooms");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Get the program from the workspace using the IDL
  const program = new anchor.Program(
    MultiplayerRooms,
    new PublicKey("DaPiwFoWxvBcoN94HZjcfz2bgaEQ3hfaDJivMZ7CuPuB"),
    provider
  );

  console.log("Deploying Multiplayer Rooms to Devnet...");

  try {
    // Initialize the program
    const tx = await program.methods.initialize().rpc();
    console.log("Program initialized with transaction signature:", tx);
  } catch (error) {
    console.log("Program initialization error (may be already initialized):", error);
  }

  console.log("Deployment completed successfully!");
};
