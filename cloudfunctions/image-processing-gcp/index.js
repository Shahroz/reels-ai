'use strict';

const { Storage } = require('@google-cloud/storage');
const path = require('path');
const os = require('os');
const fs = require('fs').promises;
const sharp = require('sharp');

const gcs = new Storage();

/**
 * Processes an image file, converting it to PNG format.
 * This is an HTTP-triggered function.
 *
 * @param {object} req The HTTP request object.
 * @param {object} res The HTTP response object.
 */
exports.processImage = async (req, res) => {
  // Set CORS headers for all requests
  res.set('Access-Control-Allow-Origin', '*');
  res.set('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.set('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-API-Token');

  // Handle preflight requests
  if (req.method === 'OPTIONS') {
    res.status(204).send('');
    return;
  }

  if (req.method !== 'POST') {
    res.status(405).send('Method Not Allowed');
    return;
  }

  // Check for API token authentication
  const apiToken = req.headers['x-api-token'] || req.headers['authorization']?.replace('Bearer ', '');
  const expectedToken = process.env.API_TOKEN;

  if (expectedToken && apiToken !== expectedToken) {
    res.status(401).send({ error: 'Unauthorized: Invalid or missing API token' });
    return;
  }

  const { imageGcsUri, outputGcsPath } = req.body;

  if (!imageGcsUri || !outputGcsPath) {
    res.status(400).send({ error: 'Missing parameters: imageGcsUri or outputGcsPath' });
    return;
  }

  const inUriParts = imageGcsUri.replace('gs://', '').split('/');
  const inBucketName = inUriParts.shift();
 const inFilePath = inUriParts.join('/');
 const inBucket = gcs.bucket(inBucketName);
 const inFileName = path.basename(inFilePath);
const tempInFilePath = path.join(os.tmpdir(), inFileName);

  const outUriParts = outputGcsPath.replace('gs://', '').split('/');
  const outBucketName = outUriParts.shift();
  const gcsOutputPath = outUriParts.join('/');

  if (!gcsOutputPath || gcsOutputPath.endsWith('/')) {
    res.status(400).send({ error: 'Invalid outputGcsPath: must be a full file path, not a directory.' });
    return;
  }

  const outFileName = path.basename(gcsOutputPath);
  const tempOutFilePath = path.join(os.tmpdir(), outFileName);

 try {
   // Download the image file.
   console.log(`Downloading image from gs://${inBucketName}/${inFilePath} to ${tempInFilePath}...`);
    await inBucket.file(inFilePath).download({ destination: tempInFilePath });
    console.log('Image downloaded to', tempInFilePath);

    // Process the image with sharp
    console.log(`Converting ${inFileName} to PNG...`);
    await sharp(tempInFilePath)
      .png({ quality: 90, compressionLevel: 9 }) // High quality, max compression
     .toFile(tempOutFilePath);
   console.log(`Image converted and saved to ${tempOutFilePath}`);

   // Upload the processed image
   const outBucket = gcs.bucket(outBucketName);

   console.log(`Uploading processed image to gs://${outBucketName}/${gcsOutputPath}...`);
   await outBucket.upload(tempOutFilePath, { destination: gcsOutputPath });
    console.log(`Uploaded processed image to ${gcsOutputPath}`);

    const outputUri = `gs://${outBucketName}/${gcsOutputPath}`;

    res.status(200).send({
      message: 'Successfully processed and uploaded image.',
      imagePath: outputUri,
    });
  } catch (error) {
    console.error('Error in processImage:', error);
    res.status(500).send({ error: 'An unexpected error occurred.', details: error.message });
  } finally {
    // Clean up local files
    try {
      await fs.unlink(tempInFilePath);
      console.log('Temporary input file removed.');
    } catch (e) {
      if (e.code !== 'ENOENT') console.error('Error removing temp input file:', e);
    }
    try {
      await fs.unlink(tempOutFilePath);
      console.log('Temporary output file removed.');
    } catch (e) {
      if (e.code !== 'ENOENT') console.error('Error removing temp output file:', e);
    }
  }
};
