import multer from 'multer';
import { STORAGE_TYPE } from '../env-vars';
import { getPresignedStorageUrl, getStorageUrl, multerFileSystemStorage, upload } from './fileSystem';
import { generatePresignedUrl, multerS3Storage, uploadStringAsFileS3 } from './s3';

export type UploadFileResponse = {
  bucket: string;
  key: string;
};

// Get the URL for a given file (key).
export const getFileUrl = async (key: string) => {
  switch (STORAGE_TYPE) {
    case 's3':
      return await generatePresignedUrl(key);
    case 'file-system':
      return getStorageUrl(key);
    default:
      throw new Error(`Unsupported storage type in getFileUrl(): ${STORAGE_TYPE}`);
  }
};

// Get a presigned URL for a given file (key).
export const getPresignedFileUrl = async (key: string) => {
  switch (STORAGE_TYPE) {
    case 's3':
      return await generatePresignedUrl(key);
    case 'file-system':
      return getPresignedStorageUrl(key);
    default:
      throw new Error(`Unsupported storage type in getPresignedFileUrl(): ${STORAGE_TYPE}`);
  }
};

// Upload a file (key).
export const uploadFile = async (key: string, contents: string, jwt: string): Promise<UploadFileResponse> => {
  switch (STORAGE_TYPE) {
    case 's3':
      return await uploadStringAsFileS3(key, contents);
    case 'file-system':
      return await upload(key, contents, jwt);
    default:
      throw new Error(`Unsupported storage type in uploadFile(): ${STORAGE_TYPE}`);
  }
};

// Multer middleware for file uploads.
export const uploadMiddleware = (): multer.Multer => {
  switch (STORAGE_TYPE) {
    case 's3':
      return multerS3Storage();
    case 'file-system':
      return multerFileSystemStorage as unknown as multer.Multer;
    default:
      throw new Error(`Unsupported storage type in uploadMiddleware(): ${STORAGE_TYPE}`);
  }
};
