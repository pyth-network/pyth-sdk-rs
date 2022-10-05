const fs = require('fs');
const web3 = require("@solana/web3.js");
const {struct, b, u8, blob} = require("@solana/buffer-layout");

export const invoke = async (loan: string, collateral: string) => {
    /* Obtain the contract keypair */
    var contract;
    try {
        let data = fs.readFileSync(
            '../build/example_sol_contract-keypair.json'
        );
        contract = web3.Keypair.fromSecretKey(
            new Uint8Array(JSON.parse(data))
        );
        console.info("Invoking contract " + contract.publicKey);
    } catch (error) {
        console.error("Please run scripts/build.sh first.");
        return;
    }

    /* Prepare the payer account */
    let conn = new web3.Connection(web3.clusterApiUrl('devnet'));
    console.info("Airdropping to the payer account...");
    let payer = web3.Keypair.generate();
    let airdropSig = await conn.requestAirdrop(
        payer.publicKey, web3.LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdropSig);

    /* Prepare the createInst instruction which creates an
     * account storing the AdminConfig data for the instructions */
    let loanInfoSize = 1 + 32 + 32;
    let dataAccount = web3.Keypair.generate();
    let dataCost = await conn.getMinimumBalanceForRentExemption(loanInfoSize);
    const createInst = web3.SystemProgram.createAccount({
        lamports: dataCost,
        space: loanInfoSize,
        programId: contract.publicKey,
        fromPubkey: payer.publicKey,
        newAccountPubkey: dataAccount.publicKey,
    });

    /* Prepare the accounts and instruction data for transactions */
    const dataKey = dataAccount.publicKey;
    const loanKey = new web3.PublicKey(loan);
    const collateralKey = new web3.PublicKey(collateral);
    let accounts =
        [{pubkey: contract.publicKey, isSigner: true, isWritable: false},
         {pubkey: dataKey, isSigner: false, isWritable: false},
         {pubkey: loanKey, isSigner: false, isWritable: false},
         {pubkey: collateralKey, isSigner: false, isWritable: false},
        ];

    let initLayout = struct([ u8('instruction') ])
    let initData = Buffer.alloc(initLayout.span);
    let loan2ValueLayout = struct([
        u8('instruction'), blob(8, 'loan_qty'), blob(8, 'collateral_qty')
    ])
    let loan2ValueData = Buffer.alloc(loan2ValueLayout.span);

    /* Invoke the Init instruction (instruction #0) */
    console.log("Creating data account and invoking Init...");
    let txInit = new web3.Transaction({ feePayer: payer.publicKey });
    initLayout.encode({instruction: 0}, initData);
    txInit.add(
        createInst,                        /* Create data account */
        new web3.TransactionInstruction({  /* Initialize data account */
            data: initData,
            keys: accounts,
            programId: contract.publicKey
        })
    );
    let txInitSig = await web3.sendAndConfirmTransaction(
        conn, txInit, [payer, dataAccount, contract]
    );
    console.log("TxHash: " + txInitSig);

    /* Invoke the Loan2Value instruction (instruction #1) */
    console.log("Checking loan to value ratio...");
    /* Encode 0x1 in big ending */
    let loan_qty = Buffer.from('0100000000000000', 'hex');
    /* Encode 0xbb8 (3000) in big ending */
    let collateral_qty = Buffer.from('b80b000000000000', 'hex');
    let txCheck = new web3.Transaction({ feePayer: payer.publicKey });
    loan2ValueLayout.encode(
        {instruction: 1,
         loan_qty: blob(8).decode(loan_qty),
         collateral_qty: blob(8).decode(collateral_qty)}
    , loan2ValueData);
    txCheck.add(
        new web3.TransactionInstruction({
            data: loan2ValueData,
            keys: accounts,
            programId: contract.publicKey
        })
    );
    let txCheckSig = await web3.sendAndConfirmTransaction(
        conn, txCheck, [payer, contract]
    );
    console.log("TxHash: " + txCheckSig);

    /* Try to invoke the Init instruction without authority */
    console.log("Trying an unauthorized invocation of Init...");
    let attacker = web3.Keypair.generate();
    accounts[0].pubkey = attacker.publicKey

    let attackerDataAccount = web3.Keypair.generate();
    const attackerCreateInst = web3.SystemProgram.createAccount({
        lamports: dataCost,
        space: loanInfoSize,
        programId: contract.publicKey,
        fromPubkey: payer.publicKey,
        newAccountPubkey: attackerDataAccount.publicKey,
    });

    let txAttacker = new web3.Transaction({ feePayer: payer.publicKey });
    txAttacker.add(
        attackerCreateInst,
        new web3.TransactionInstruction({
            data: initData,
            keys: accounts,
            programId: contract.publicKey
        })
    );

    try {
        let txAttackerSig = await web3.sendAndConfirmTransaction(
            conn, txAttacker, [payer, attackerDataAccount, attacker]
        );
        console.error("Attacker succeeded with TxHash: " + txAttackerSig);
    } catch (error) {
        console.log("Attacker failed to invoke unauthorized Init.");
    }
}

/* Pyth price accounts on the solana devnet */
let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";
invoke(ethToUSD, usdtToUSD);
