const SOH = 0x01;
const EOT = 0x04;
const ACK = 0x06;
const NAK = 0x15;
const CRC = 0x43;

const BLOCK_SIZE = 128;

const log = (...args) => {
    document.getElementById('log').textContent += args.join(' ') + "\n";
    console.log(...args);
};

export const xmodem = async (writer, reader, file) => {

    const data = new Uint8Array(await file.arrayBuffer());

    let blockNumber = 1;

    // Wait for receiver to send 'C' (for CRC mode)
    log('Waiting for receiver to send "C"...');
    while (true) {
        const { value } = await reader.read();
        if (value && value[0] === CRC) {
            log('Receiver requested XMODEM CRC mode');
            break;
        }
    }

    // Split file into blocks
    const totalBlocks = Math.ceil(data.length / BLOCK_SIZE);

    for (let i = 0; i < totalBlocks; i++) {
        const block = data.slice(i * BLOCK_SIZE, (i + 1) * BLOCK_SIZE);
        const paddedBlock = new Uint8Array(BLOCK_SIZE);
        paddedBlock.set(block);

        const packet = new Uint8Array(3 + BLOCK_SIZE + 2); // header + data + crc
        packet[0] = SOH;
        packet[1] = blockNumber & 0xFF;
        packet[2] = 0xFF - packet[1];
        packet.set(paddedBlock, 3);

        const crc = calculateCRC(paddedBlock);
        packet[BLOCK_SIZE + 3] = (crc >> 8) & 0xFF;
        packet[BLOCK_SIZE + 4] = crc & 0xFF;

        log(`Sending block ${blockNumber}...`);
        await writer.write(packet);

        while (true) {
            const { value } = await reader.read();
            if (value && value[0] === ACK) {
            log(`Block ${blockNumber} acknowledged`);
            blockNumber++;
            break;
            } else if (value && value[0] === NAK) {
                log(`Block ${blockNumber} rejected, resending...`);
                await writer.write(packet);
            }
        }
    }

    // Send EOT
    log("Sending EOT...");
    await writer.write(new Uint8Array([EOT]));

    while (true) {
        const { value } = await reader.read();
        if (value && value[0] === ACK) {
            log("EOT acknowledged. Transfer complete.");
            break;
        }
    }

};

function calculateCRC(data) {
    let crc = 0;
    for (let i = 0; i < data.length; i++) {
        crc ^= (data[i] << 8);
        for (let j = 0; j < 8; j++) {
            if ((crc & 0x8000) !== 0) {
            crc = (crc << 1) ^ 0x1021;
            } else {
            crc = crc << 1;
            }
        }
    }
    return crc & 0xFFFF;
}
