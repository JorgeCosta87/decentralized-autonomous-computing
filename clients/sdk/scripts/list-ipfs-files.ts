#!/usr/bin/env node
/**
 * Simple script to list and access all IPFS files
 * 
 * Usage:
 *   npx tsx scripts/list-ipfs-files.ts
 *   or
 *   npm run list-ipfs
 */

import { IpfsClient } from '../src/ipfsClient.js';

const IPFS_URL = process.env.IPFS_URL || 'http://localhost:5001';

async function main() {
  const ipfsClient = new IpfsClient({ apiUrl: IPFS_URL });

  console.log('🔍 IPFS Files Tracker\n');
  console.log(`IPFS API: ${IPFS_URL}\n`);

  try {
    // List pinned files
    const pinnedFiles = await ipfsClient.listPinnedWithDetails();
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log(`📌 Pinned Files (${pinnedFiles.length})`);
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

    if (pinnedFiles.length === 0) {
      console.log('  No pinned files found.\n');
    } else {
      pinnedFiles.forEach((file, index) => {
        console.log(`${index + 1}. ${file.cid}`);
        console.log(`   🌐 Gateway: ${file.gatewayUrl}`);
        console.log(`   🔗 API:     ${file.apiUrl}\n`);
      });
    }

    // List MFS files (visible in WebUI)
    try {
      const mfsFiles = await ipfsClient.listMfsFiles();
      console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
      console.log(`📁 MFS Files (${mfsFiles.length}) - Visible in WebUI`);
      console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');

      if (mfsFiles.length === 0) {
        console.log('  No files in MFS. Files uploaded via API are automatically added.\n');
      } else {
        mfsFiles.forEach((file: any, index: number) => {
          console.log(`${index + 1}. ${file.Name} (${file.Type})`);
          if (file.Size) {
            console.log(`   Size: ${file.Size} bytes`);
          }
        });
        console.log('\n');
      }
    } catch (error: any) {
      console.log('⚠️  Could not list MFS files:', error.message);
      console.log('   (Files are still accessible via CID)\n');
    }

    // Quick access links
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
    console.log('🔗 Quick Access');
    console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n');
    console.log('  WebUI:        http://localhost:5001/webui');
    console.log('  WebUI Files:  http://localhost:5001/webui → Files → dac-uploads');
    console.log('  Gateway:      http://localhost:8080/ipfs/<CID>');
    console.log('  API:          http://localhost:5001/api/v0/cat?arg=<CID>\n');

  } catch (error: any) {
    console.error('❌ Error:', error.message);
    process.exit(1);
  }
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
