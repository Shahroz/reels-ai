'use strict';

const { Storage } = require('@google-cloud/storage');
const path = require('path');
const os = require('os');
const fs = require('fs');
const ffmpeg = require('fluent-ffmpeg');
const ffmpeg_static = require('ffmpeg-static');

const gcs = new Storage();

// Makes an ffmpeg command return a promise.
function promisifyCommand(command) {
  return new Promise((resolve, reject) => {
    command.on('end', resolve).on('error', reject).run();
  });
}

/**
 * Extracts frames from a video file at specified timestamps.
 * This is an HTTP-triggered function.
 *
 * @param {object} req The HTTP request object.
 * @param {object} res The HTTP response object.
 */
exports.extractVideoFrames = async (req, res) => {
  // Set CORS headers for all requests
  res.set('Access-Control-Allow-Origin', '*');
  res.set('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.set('Access-Control-Allow-Headers', 'Content-Type');

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

  const { videoGcsUri, timestamps, outputGcsPath } = req.body;

  if (!videoGcsUri || !timestamps || !outputGcsPath) {
    res.status(400).send('Missing parameters: videoGcsUri, timestamps, or outputGcsPath');
    return;
  }

  if (!Array.isArray(timestamps) || timestamps.length === 0) {
  res.status(400).send('timestamps must be a non-empty array of numbers in milliseconds.');
  return;
}
  const uriParts = videoGcsUri.replace('gs://', '').split('/');
  const bucketName = uriParts.shift();
  const filePath = uriParts.join('/');
  const bucket = gcs.bucket(bucketName);
  const fileName = path.basename(filePath);
  const tempFilePath = path.join(os.tmpdir(), fileName);

  try {
    // Download the video file.
    console.log(`Downloading video from gs://${bucketName}/${filePath} to ${tempFilePath}...`);
    await bucket.file(filePath).download({ destination: tempFilePath });
    console.log('Video downloaded to', tempFilePath);

    const outputUriParts = outputGcsPath.replace('gs://', '').split('/');
    const outputBucketName = outputUriParts.shift();
    const outputFolderPath = outputUriParts.join('/');
   const outputBucket = gcs.bucket(outputBucketName);

    const frameProcessingResults = await Promise.allSettled(timestamps.map(async (timestamp, index) => {
      const totalSeconds = Math.floor(timestamp / 1000);
      const hours = Math.floor(totalSeconds / 3600).toString().padStart(2, '0');
      const minutes = Math.floor((totalSeconds % 3600) / 60).toString().padStart(2, '0');
      const seconds = (totalSeconds % 60).toString().padStart(2, '0');
      const frameNumber = index + 1;

      const outputFileName = `_${hours}_${minutes}_${seconds}_frame_${frameNumber}.png`;
      const tempOutputFilePath = path.join(os.tmpdir(), outputFileName);

      try {
        console.log(`Extracting frame for timestamp ${timestamp}ms...`);
        const command = ffmpeg(tempFilePath)
          .setFfmpegPath(ffmpeg_static)
          .seekInput(timestamp / 1000) // seek to timestamp in seconds
          .frames(1)
          .output(tempOutputFilePath);

        await promisifyCommand(command);
        console.log(`Frame extracted at ${timestamp}ms to ${tempOutputFilePath}`);

        if (!fs.existsSync(tempOutputFilePath)) {
          throw new Error(`ffmpeg command completed but output file was not created for timestamp ${timestamp}.`);
        }

        const gcsOutputPath = path.join(outputFolderPath, outputFileName);
        console.log(`Uploading frame to gs://${outputBucketName}/${gcsOutputPath}...`);
        await outputBucket.upload(tempOutputFilePath, { destination: gcsOutputPath });
        console.log(`Uploaded frame to ${gcsOutputPath}`);

        return `gs://${outputBucketName}/${gcsOutputPath}`;
      } finally {
        // Clean up local frame file
        if (fs.existsSync(tempOutputFilePath)) {
          fs.unlinkSync(tempOutputFilePath);
        }
      }
    }));

    const successfulFrames = [];
    const failedFrames = [];

    frameProcessingResults.forEach((result, index) => {
      if (result.status === 'fulfilled') {
        successfulFrames.push(result.value);
      } else {
        const reason = result.reason.message || result.reason.toString();
        console.error(`Failed to process frame for timestamp ${timestamps[index]}:`, reason);
        failedFrames.push({ timestamp: timestamps[index], error: reason });
      }
    });

    if (successfulFrames.length === 0 && failedFrames.length > 0) {
      return res.status(500).send({
        message: 'Failed to extract any frames.',
        failures: failedFrames
      });
    }

    if (failedFrames.length > 0) {
      return res.status(207).send({
        message: `Successfully extracted ${successfulFrames.length} of ${timestamps.length} frames.`,
        framePaths: successfulFrames,
        failures: failedFrames
      });
    }

    res.status(200).send({
      message: `Successfully extracted and uploaded ${timestamps.length} frames.`,
      framePaths: successfulFrames
    });

  } catch (error) {
    console.error('Error in extractVideoFrames:', error);
    res.status(500).send({ error: 'An unexpected error occurred.', details: error.message });
  } finally {
    // Clean up local video file
    if (fs.existsSync(tempFilePath)) {
      fs.unlinkSync(tempFilePath);
      console.log('Temporary video file removed.');
    }
  }
};
