const fs = require('fs');
const assert = require("assert");
const anchor = require("@project-serum/anchor");

let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const config = anchor.web3.Keypair.generate();
const program = anchor.workspace.ExampleSolAnchorContract;
let programId = program.programId;
var programKey;
try {
    let data = fs.readFileSync(
        'program_address.json'
    );
    programKey = anchor.web3.Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(data))
    );
} catch (error) {
    throw new Error("Please make sure the program key is program_address.json.");
}

try {
    assert(programId.equals(programKey.publicKey));
} catch (error) {
    throw new Error("Please make sure you have the same program address inAnchor.toml and program_address.json");
}

it("Initialize the config.", async () => {
    let txSig = await program.rpc.init(
        {
            isInitialized: true,
            loanPriceFeedId: new anchor.web3.PublicKey(ethToUSD),
            collateralPriceFeedId: new anchor.web3.PublicKey(usdtToUSD),
        },
        {
            accounts: {
                program: programId,
                payer: provider.wallet.publicKey,
                config: config.publicKey,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
            signers: [config, programKey],
        });

    console.log("Config key: " + config.publicKey);
    console.log("Init() is invoked: " + txSig);
});

it("Check loan to value ratio.", async () => {
    let txSig = await program.rpc.loanToValue(
        new anchor.BN(1),
        new anchor.BN(3000),
        {
            accounts: {
                config: config.publicKey,
                pythLoanAccount: new anchor.web3.PublicKey(ethToUSD),
                pythCollateralAccount: new anchor.web3.PublicKey(usdtToUSD),
            }
        }
    );

    console.log("Loan2Value() is invoked: " + txSig);
});

it("Prevent initialization of config without authority.", async () => {
    const attacker = anchor.web3.Keypair.generate();

    var txSig, success = false;
    try {
        txSig = await program.rpc.init(
            {
                isInitialized: true,
                loanPriceFeedId: new anchor.web3.PublicKey(ethToUSD),
                collateralPriceFeedId: new anchor.web3.PublicKey(usdtToUSD),
            },
            {
                accounts: {
                    program: programId,
                    payer: provider.wallet.publicKey,
                    config: config.publicKey,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [config, attacker],
            });
        success = true;
    } catch (error) {
        console.log("Attacker failed at unauthorized init");
    }
    if (success)
        throw new Error("Attacker succeeded! TxHash: " + txSig);
});
