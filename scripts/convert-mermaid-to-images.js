const fs = require('fs');
const path = require('path');
const https = require('https');

const diagramsDir = path.join(__dirname, '../docs/diagrams');
const outputDir = path.join(__dirname, '../docs/diagrams/images');

// Create output directory if it doesn't exist
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Extract mermaid diagrams from markdown
function extractMermaidDiagrams(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const mermaidBlocks = [];
  const regex = /```mermaid\n([\s\S]*?)```/g;
  let match;
  let index = 0;
  
  while ((match = regex.exec(content)) !== null) {
    mermaidBlocks.push({
      content: match[1].trim(),
      index: index++
    });
  }
  
  return mermaidBlocks;
}

// Convert mermaid to image using mermaid.ink API with retries
function convertToImage(mermaidContent, outputPath, retries = 3) {
  return new Promise((resolve, reject) => {
    const attempt = (attemptNum) => {
      // Use base64url encoding (URL-safe base64)
      const encoded = Buffer.from(mermaidContent)
        .toString('base64')
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=/g, '');
      
      const url = `https://mermaid.ink/img/${encoded}`;
      
      const file = fs.createWriteStream(outputPath);
      
      https.get(url, (response) => {
        if (response.statusCode === 503 || response.statusCode === 500) {
          file.close();
          if (fs.existsSync(outputPath)) {
            fs.unlinkSync(outputPath);
          }
          if (attemptNum < retries) {
            console.log(`    Retrying... (${attemptNum + 1}/${retries})`);
            setTimeout(() => attempt(attemptNum + 1), 2000);
            return;
          }
          reject(new Error(`Failed after ${retries} attempts: ${response.statusCode} ${response.statusMessage}`));
          return;
        }
        
        if (response.statusCode !== 200) {
          file.close();
          if (fs.existsSync(outputPath)) {
            fs.unlinkSync(outputPath);
          }
          reject(new Error(`Failed to fetch image: ${response.statusCode} ${response.statusMessage}`));
          return;
        }
        
        // Verify content type
        const contentType = response.headers['content-type'];
        if (!contentType || !contentType.startsWith('image/')) {
          file.close();
          if (fs.existsSync(outputPath)) {
            fs.unlinkSync(outputPath);
          }
          reject(new Error(`Invalid content type: ${contentType}`));
          return;
        }
        
        response.pipe(file);
        
        file.on('finish', () => {
          file.close();
          // Verify file is not empty
          const stats = fs.statSync(outputPath);
          if (stats.size === 0) {
            fs.unlinkSync(outputPath);
            if (attemptNum < retries) {
              console.log(`    Retrying... (${attemptNum + 1}/${retries})`);
              setTimeout(() => attempt(attemptNum + 1), 2000);
              return;
            }
            reject(new Error('Downloaded file is empty'));
            return;
          }
          resolve();
        });
      }).on('error', (err) => {
        if (fs.existsSync(outputPath)) {
          fs.unlinkSync(outputPath);
        }
        if (attemptNum < retries) {
          console.log(`    Retrying... (${attemptNum + 1}/${retries})`);
          setTimeout(() => attempt(attemptNum + 1), 2000);
          return;
        }
        reject(err);
      });
    };
    
    attempt(1);
  });
}

// Process all files
async function processAll() {
  const markdownFiles = fs.readdirSync(diagramsDir)
    .filter(file => file.endsWith('.md'))
    .map(file => path.join(diagramsDir, file));

  for (const filePath of markdownFiles) {
    const fileName = path.basename(filePath, '.md');
    const diagrams = extractMermaidDiagrams(filePath);
    
    if (diagrams.length === 0) {
      console.log(`No mermaid diagrams found in ${fileName}`);
      continue;
    }
    
    console.log(`Processing ${fileName}: ${diagrams.length} diagram(s)`);
    
    for (let idx = 0; idx < diagrams.length; idx++) {
      const diagram = diagrams[idx];
      const outputFileName = diagrams.length === 1 
        ? `${fileName}.png`
        : `${fileName}-${idx + 1}.png`;
      const outputPath = path.join(outputDir, outputFileName);
      
      console.log(`  Converting to ${outputFileName}...`);
      try {
        await convertToImage(diagram.content, outputPath);
        console.log(`  ✓ Success`);
      } catch (error) {
        console.error(`  ✗ Error: ${error.message}`);
      }
    }
  }
  
  console.log('\nAll diagrams processed!');
}

processAll().catch(console.error);
